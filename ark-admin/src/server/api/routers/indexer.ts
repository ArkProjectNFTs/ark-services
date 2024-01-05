import { DynamoDB, type AttributeValue } from "@aws-sdk/client-dynamodb";
import { ECSClient } from "@aws-sdk/client-ecs";
import { z } from "zod";

import { runTask } from "~/lib/awsTasksSpawner";
import { fetchBlocks } from "~/lib/fetchBlocks";
import { fetchLastBlock } from "~/lib/fetchLastBlock";
import { fetchLatestBlocks } from "~/lib/fetchLastestBlocks";
import { type Network } from "~/types";
import { createTRPCRouter, protectedProcedure } from "../trpc";

type IndexerTask = {
  indexationProgress: number;
  taskId: string;
  status: string;
  from: number;
  to: number;
  forceMode: boolean;
  version?: string;
  updatedAt?: string;
  createdAt?: string;
  currentBlockNumber?: string;
};

const ECS_CLUSTER =
  "arn:aws:ecs:us-east-1:223605539824:cluster/ArkStack-staging-Indexers1FAD4BAF-XuWWuGXgYkqe";
const AWS_REGION = "us-east-1";

const client = new ECSClient({
  region: AWS_REGION,
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID!,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY!,
  },
});

const dynamodb = new DynamoDB({ region: AWS_REGION });

const fetchTasks = async (
  network: Network,
): Promise<Record<string, AttributeValue>[]> => {
  const dynamoResult = await dynamodb.query({
    TableName: `${process.env.TABLE_NAME_PREFIX}${network}`,
    IndexName: "GSI1PK-GSI1SK-index",
    KeyConditionExpression: "#GSI1PK = :GSI1PK",
    ExpressionAttributeNames: { "#GSI1PK": "GSI1PK" },
    ExpressionAttributeValues: { ":GSI1PK": { S: "INDEXER" } },
    ScanIndexForward: false,
  });

  // const command = new ListTasksCommand({ cluster: ECS_CLUSTER });
  // const ecsOutput = await client.send(command);
  return dynamoResult.Items ?? [];
};

const mapDynamoItem = (item: Record<string, AttributeValue>): IndexerTask => {
  const regex = /TASK#([a-fA-F0-9]+)/;
  const match = item.SK?.S?.match(regex);
  const taskId = match?.[1] ?? "";
  return {
    indexationProgress: item.Data?.M?.IndexationProgress?.N
      ? parseInt(item.Data.M.IndexationProgress.N.toString())
      : 0,
    taskId,
    status: item.Data?.M?.Status?.S ?? "",
    from: item.Data?.M?.From?.N ? parseInt(item.Data.M.From.N.toString()) : 0,
    to: item.Data?.M?.To?.N ? parseInt(item.Data.M.To.N.toString()) : 0,
    version: item?.Data?.M?.Version?.S,
    updatedAt: item?.Data?.M?.LastUpdate?.N,
    createdAt: item?.Data?.M?.CreatedAt?.N,
    forceMode: item?.Data?.M?.ForceMode?.BOOL ?? false,
    currentBlockNumber: item?.Data?.M?.CurrentBlockNumber?.N,
  };
};

export const indexerRouter = createTRPCRouter({
  latestBlocks: protectedProcedure
    .input(z.object({ network: z.enum(["testnet", "mainnet"]) }))
    .query(async ({ input }) => {
      const results = await fetchLatestBlocks(input.network);
      return results.map((r) => {
        const sk = r.GSI1SK?.S ?? "";
        const ts = sk.split("#")[1];

        return {
          blockId: r.Data?.M?.BlockNumber?.N ?? "",
          timestamp: ts ? parseInt(ts, 10) : 0,
        };
      });
    }),

  allBlocks: protectedProcedure
    .input(z.object({ network: z.enum(["testnet", "mainnet"]) }))
    .query(async ({ input }) => {
      const latest = await fetchLastBlock(input.network);
      const { ranges, rangeSize, count } = await fetchBlocks(
        input.network,
        latest,
      );

      return {
        latest,
        ranges,
        rangeSize,
        count,
      };
    }),
  allTasks: protectedProcedure
    .input(z.object({ network: z.enum(["testnet", "mainnet"]) }))
    .query(async ({ input }: { input: { network: Network } }) => {
      try {
        const tasks = await fetchTasks(input.network);

        return tasks.reduce<IndexerTask[]>((acc, task) => {
          if (task.Data?.M) {
            const item = mapDynamoItem(task);

            return acc.concat(item);
          }
          return acc;
        }, []);
      } catch (error) {
        console.error(error);
        return [];
      }
    }),

  deleteTask: protectedProcedure
    .input(
      z.object({
        taskId: z.string(),
        network: z.enum(["testnet", "mainnet"]),
      }),
    )
    .mutation(async ({ input }) => {
      try {
        await dynamodb.deleteItem({
          Key: {
            PK: { S: "INDEXER" },
            SK: { S: `TASK#${input.taskId}` },
          },
          TableName: `ark_project_${input.network}`,
        });
      } catch (error) {
        console.error(error);
      }
    }),

  spawnTasks: protectedProcedure
    .input(
      z.object({
        from: z.number().min(0),
        to: z.number().min(0),
        numberOfTasks: z.number().min(1),
        network: z.enum(["testnet", "mainnet"]),
        forceMode: z.boolean().optional(),
        logLevel: z.string().optional(),
      }),
    )
    .mutation(async ({ input }) => {
      console.log("spawnTasks() input:", input);

      // ArkStack-staging-Indexers1FAD4BAF-XuWWuGXgYkqe

      const rangeSize = Math.floor(
        (input.to - input.from + 1) / input.numberOfTasks,
      );
      const subnetId =
        input.network === "mainnet"
          ? "subnet-0c28889f016ad63f5"
          : "subnet-05ebee80f9f4299a5";

      const taskDefinition =
        input.network === "mainnet"
          ? "ArkStackstagingIndexerMainnetTaskDefinition701D90CD"
          : "ArkStackstagingIndexerTestnetTaskDefinitionD104DD87";
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
            logLevel: input.logLevel ?? "info",
            forceMode: input.forceMode ?? false,
          };
          const commandOutput = await runTask(client, commandOptions);

          console.log("=> commandOutput", commandOutput);

          for (const task of commandOutput.tasks ?? []) {
            if (task.taskArn) {
              const taskId = task.taskArn.split("/").pop();
              const creationDate = Math.floor(Date.now() / 1000);

              const putRequest = await dynamodb.putItem({
                TableName: `ark_project_${input.network}`,
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
                      ForceMode: { BOOL: input.forceMode ?? false },
                    },
                  },
                },
              });

              console.log("=> putRequest", putRequest);
            }
          }
        }
      } catch (error) {
        console.error(error);
      }
    }),
});
