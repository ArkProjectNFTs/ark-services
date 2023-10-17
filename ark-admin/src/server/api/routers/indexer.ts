import { DynamoDB, type AttributeValue } from "@aws-sdk/client-dynamodb";
import {
  ECSClient,
  ListTasksCommand,
  type ListTasksCommandOutput,
} from "@aws-sdk/client-ecs";
import { z } from "zod";

import { runTask } from "~/lib/awsTasksSpawner";
import { createTRPCRouter, protectedProcedure } from "../trpc";

const ECS_CLUSTER = "arn:aws:ecs:us-east-1:223605539824:cluster/ark-indexers";
const AWS_REGION = "us-east-1";

const client = new ECSClient({
  region: AWS_REGION,
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID!,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY!,
  },
});

const dynamodb = new DynamoDB({ region: AWS_REGION });

type Network = "testnet" | "mainnet";

const getTableName = (network: Network): string => {
  return network === "mainnet" ? "ark_project_mainnet" : "ark_project_testnet";
};

const fetchTasks = async (network: Network) => {
  const dynamoResult = await dynamodb.query({
    TableName: getTableName(network),
    IndexName: "GSI1PK-GSI1SK-index",
    KeyConditionExpression: "#GSI1PK = :GSI1PK",
    ExpressionAttributeNames: { "#GSI1PK": "GSI1PK" },
    ExpressionAttributeValues: { ":GSI1PK": { S: "INDEXER" } },
    ScanIndexForward: false,
  });

  const command = new ListTasksCommand({ cluster: ECS_CLUSTER });
  const ecsOutput = await client.send(command);
  return { dynamoResult, ecsOutput };
};

const mapDynamoItem = (
  item: Record<string, AttributeValue>,
  ecsOutput: ListTasksCommandOutput,
) => {
  const regex = /TASK#([a-fA-F0-9]+)/;
  const match = item.SK?.S?.match(regex);
  const taskId = match?.[1] ?? "";
  const isRunning = ecsOutput.taskArns?.some((arn: string) =>
    arn.includes(taskId),
  );

  return {
    indexationProgress: item.Data?.M?.IndexationProgress?.N
      ? parseInt(item.Data.M.IndexationProgress.N.toString())
      : 0,
    taskId,
    isRunning,
    from: item.Data?.M?.From?.N ? parseInt(item.Data.M.From.N.toString()) : 0,
    to: item.Data?.M?.To?.N ? parseInt(item.Data.M.To.N.toString()) : 0,
    version: item?.Data?.M?.Version?.S,
    updatedAt: item?.Data?.M?.LastUpdate?.N,
    createdAt: item?.Data?.M?.CreatedAt?.N,
  };
};

export const indexerRouter = createTRPCRouter({
  allTasks: protectedProcedure
    .input(z.object({ network: z.enum(["testnet", "mainnet"]) }))
    .query(async ({ input }: { input: { network: Network } }) => {
      try {
        const { dynamoResult, ecsOutput } = await fetchTasks(input.network);
        const result =
          dynamoResult.Items?.reduce<
            {
              indexationProgress: number;
              taskId: string;
              isRunning: boolean | undefined;
              from: number;
              to: number;
              version: string | undefined;
              updatedAt: string | undefined;
              createdAt: string | undefined;
            }[]
          >((acc, item) => {
            if (item.Data?.M) {
              return acc.concat(mapDynamoItem(item, ecsOutput));
            }
            return acc;
          }, []) ?? [];

        return result;
      } catch (error) {
        console.error(error);
        return [];
      }
    }),

  spawnTasks: protectedProcedure
    .input(
      z.object({
        from: z.number().min(0),
        to: z.number().min(0),
        numberOfTasks: z.number().min(1),
        network: z.enum(["testnet", "mainnet"]),
      }),
    )
    .mutation(
      async ({
        input,
      }: {
        input: {
          from: number;
          to: number;
          numberOfTasks: number;
          network: Network;
        };
      }) => {
        const rangeSize = Math.floor(
          (input.to - input.from + 1) / input.numberOfTasks,
        );
        const subnetId =
          input.network === "mainnet"
            ? "subnet-0c28889f016ad63f5"
            : "subnet-05ebee80f9f4299a5";
        const taskDefinition =
          input.network === "mainnet"
            ? "ark-indexer-task-mainnet"
            : "ark-indexer-task-testnet";
        try {
          for (let i = 0; i < input.numberOfTasks; i++) {
            const subFrom = input.from + rangeSize * i;
            const subTo = Math.min(subFrom + rangeSize - 1, input.to);

            const commandOptions = {
              cluster: ECS_CLUSTER,
              network: input.network,
              from: subFrom,
              to: subTo,
              subnetId,
              taskDefinition,
            };
            const commandOutput = await runTask(client, commandOptions);

            console.log("=> commandOutput", commandOutput);

            if (commandOutput.tasks?.[0]?.taskArn) {
              const taskId = commandOutput.tasks[0].taskArn.split("/").pop();
              const creationDate = Math.floor(Date.now() / 1000);

              const putRequest = await dynamodb.putItem({
                TableName: getTableName(input.network),
                Item: {
                  PK: { S: "INDEXER" },
                  SK: { S: `TASK#${taskId}` },
                  GSI1PK: { S: "INDEXER" },
                  GSI1SK: { S: creationDate.toString() },
                  Data: {
                    M: {
                      From: { N: subFrom.toString() },
                      To: { N: subTo.toString() },
                      Status: { S: "requested" },
                      IndexationProgress: { N: "0" },
                      LastUpdate: { N: creationDate.toString() },
                      CreatedAt: { N: creationDate.toString() },
                    },
                  },
                },
              });

              console.log("=> putRequest", putRequest);
            }
          }
        } catch (error) {
          console.error(error);
        }
      },
    ),
});
