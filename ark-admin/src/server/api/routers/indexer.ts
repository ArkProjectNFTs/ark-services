import { DynamoDB, type AttributeValue } from "@aws-sdk/client-dynamodb";
import { ECSClient } from "@aws-sdk/client-ecs";
import { z } from "zod";

import { runTask } from "~/lib/awsTasksSpawner";
import { fetchLastBlock } from "~/lib/fetchLastBlock";
import { splitIntoRanges } from "~/lib/splitIntoRanges";
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

function getRandomNumber(min: number, max: number) {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function getRandomNumbers(n: number, min: number, max: number) {
  const numbers: number[] = [];

  for (let i = 0; i < n; i++) {
    numbers.push(getRandomNumber(min, max));
  }

  return numbers.sort((a, b) => a - b);
}

const getTableName = (network: Network): string => {
  return network === "mainnet" ? "ark_project_mainnet" : "ark_project_testnet";
};

const fetchTasks = async (
  network: Network,
): Promise<Record<string, AttributeValue>[]> => {
  const dynamoResult = await dynamodb.query({
    TableName: getTableName(network),
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
    forceMode: item?.Data?.M?.ForceMode?.BOOL,
  };
};

type IndexerTask = {
  indexationProgress: number;
  taskId: string;
  status: string;
  from: number;
  to: number;
  version?: string;
  updatedAt?: string;
  createdAt?: string;
  forceMode?: boolean;
};

function containsNumbersInRange(arr: number[], start: number, end: number) {
  const numSet = new Set(arr);

  for (let i = start; i <= end; i++) {
    if (numSet.has(i)) {
      return true;
    }
  }

  return false;
}

function numbersInRange(arr: number[], start: number, end: number) {
  const numSet = new Set(arr);
  const result: number[] = [];

  for (let i = start; i <= end; i++) {
    if (numSet.has(i)) {
      result.push(i);
    }
  }

  return result;
}

export const indexerRouter = createTRPCRouter({
  allBlocks: protectedProcedure
    .input(z.object({ network: z.enum(["testnet", "mainnet"]) }))
    .query(async ({ input }) => {
      const latest = await fetchLastBlock(input.network);
      const blocks = getRandomNumbers(500, 24, 100000);
      const ranges = splitIntoRanges(latest, 120).map(([start, end]) => ({
        start,
        end,
        hasUnindexed: containsNumbersInRange(blocks, start!, end!),
        blocks: numbersInRange(blocks, start!, end!),
      }));

      return {
        blocks,
        latest,
        ranges,
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
          TableName: getTableName(input.network),
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
            logLevel: input.logLevel ?? "info",
          };
          const commandOutput = await runTask(client, commandOptions);

          console.log("=> commandOutput", commandOutput);

          for (const task of commandOutput.tasks ?? []) {
            if (task.taskArn) {
              const taskId = task.taskArn.split("/").pop();
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
