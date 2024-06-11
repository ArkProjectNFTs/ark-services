import { DynamoDB, type AttributeValue } from "@aws-sdk/client-dynamodb";
import { ECSClient } from "@aws-sdk/client-ecs";
import { z } from "zod";

import { runTask } from "~/lib/awsTasksSpawner";
import { fetchBlocks, fetchLatestBlocks } from "~/lib/fetchBlocks";
import { fetchLastBlock } from "~/lib/fetchLastBlock";
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
  // const dynamoResult = await dynamodb.query({
  //   TableName: getTableName(network),
  //   IndexName: "GSI1PK-GSI1SK-index",
  //   KeyConditionExpression: "#GSI1PK = :GSI1PK",
  //   ExpressionAttributeNames: { "#GSI1PK": "GSI1PK" },
  //   ExpressionAttributeValues: { ":GSI1PK": { S: "INDEXER" } },
  //   ScanIndexForward: false,
  // });

  // // const command = new ListTasksCommand({ cluster: ECS_CLUSTER });
  // // const ecsOutput = await client.send(command);
  // return dynamoResult.Items ?? [];

  return Promise.resolve([]);
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
  metadata: protectedProcedure
    .input(
      z.object({
        network: z.enum([
          "production-sepolia",
          "production-mainnet",
          "staging-sepolia",
          "staging-mainnet",
        ]),
      }),
    )
    .query(async ({ input }: { input: { network: Network } }) => {
      const contracts = {};
      // let lastEvaluatedKey;
      const totalCount = 0;

      // do {
      //   const results = await dynamodb.query({
      //     TableName: getTableName(input.network),
      //     IndexName: "GSI5PK-GSI5SK-index",
      //     KeyConditionExpression: "#GSI5PK = :GSI5PK",
      //     ExpressionAttributeNames: { "#GSI5PK": "GSI5PK" },
      //     ExpressionAttributeValues: {
      //       ":GSI5PK": { S: "METADATA#TO_REFRESH" },
      //     },
      //     ExclusiveStartKey: lastEvaluatedKey,
      //   });

      //   for (const item of results.Items) {
      //     const data = item.Data?.M;
      //     const tokenId = data.TokenId.S;

      //     if (data?.ContractAddress?.S) {
      //       const contractAddress: string = data.ContractAddress.S;
      //       const contract = contracts[contractAddress] ?? { items: [] };
      //       contract.items = [...contract.items, tokenId];
      //       contracts[contractAddress] = contract;
      //     }

      //     totalCount += 1;
      //   }

      //   lastEvaluatedKey = results.LastEvaluatedKey;
      // } while (!!lastEvaluatedKey);

      // for (const contractAddress of Object.keys(contracts)) {
      //   const contractResult = await dynamodb.query({
      //     TableName: "ark_project_mainnet",
      //     KeyConditionExpression: "#PK = :PK",
      //     ExpressionAttributeNames: { "#PK": "PK" },
      //     ExpressionAttributeValues: {
      //       ":PK": { S: `CONTRACT#${contractAddress}` },
      //     },
      //     Limit: 1,
      //   });

      //   const contractData = contractResult.Items?.[0]?.Data.M;
      //   contracts[contractAddress].name = contractData?.Name?.S;
      //   contracts[contractAddress].symbol = contractData?.Symbol?.S;
      //   contracts[contractAddress].type = contractData?.ContractType?.S;
      // }

      return Promise.resolve({ contracts, totalCount });
    }),

  latestBlocks: protectedProcedure
    .input(
      z.object({
        network: z.enum([
          "production-sepolia",
          "production-mainnet",
          "staging-sepolia",
          "staging-mainnet",
        ]),
      }),
    )
    .query(async ({ input }) => {
      const latestBlocks = await fetchLatestBlocks();

      const items = latestBlocks.map((latestBlock) => {
        console.log("latestBlock", latestBlock.block_number);
        return {
          blockId: latestBlock.block_number,
          timestamp: latestBlock.block_timestamp,
        };
      });

      return items;
    }),

  allBlocks: protectedProcedure
    .input(
      z.object({
        network: z.enum([
          "production-sepolia",
          "production-mainnet",
          "staging-sepolia",
          "staging-mainnet",
        ]),
      }),
    )
    .query(async ({ input }) => {
      const latest = await fetchLastBlock(input.network);

      console.log("Latest block:", latest);

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
    .input(
      z.object({
        network: z.enum([
          "production-sepolia",
          "production-mainnet",
          "staging-sepolia",
          "staging-mainnet",
        ]),
      }),
    )
    .query(async ({ input }: { input: { network: Network } }) => {
      try {
        const tasks = await fetchTasks(input.network);

        console.log("=> tasks", tasks, input.network);

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
        network: z.enum([
          "production-sepolia",
          "production-mainnet",
          "staging-mainnet",
          "staging-sepolia",
        ]),
      }),
    )
    .mutation(async ({ input }) => {
      // try {
      //   const tableName = getTableName(input.network);
      //   await dynamodb.deleteItem({
      //     Key: {
      //       PK: { S: "INDEXER" },
      //       SK: { S: `TASK#${input.taskId}` },
      //     },
      //     TableName: tableName,
      //   });
      // } catch (error) {
      //   console.error(error);
      // }
    }),

  spawnTasks: protectedProcedure
    .input(
      z.object({
        from: z.number().min(0),
        to: z.number().min(0),
        numberOfTasks: z.number().min(1),
        network: z.enum([
          "production-sepolia",
          "production-mainnet",
          "staging-mainnet",
          "staging-sepolia",
        ]),
        forceMode: z.boolean().optional(),
        logLevel: z.string().optional(),
      }),
    )
    .mutation(async ({ input }) => {
      const tableName = `${process.env.TABLE_NAME_PREFIX}${input.network}`;

      const rangeSize = Math.floor(
        (input.to - input.from + 1) / input.numberOfTasks,
      );

      const INDEXER_ECS_CLUSTER = process.env.ARN_ECS_INDEXER_CLUSTER ?? "";
      const INDEXER_TASK_DEF = process.env.INDEXER_TASK_DEFINITION ?? "";

      const subnetsStr = process.env.INDEXER_SUBNETS ?? "";
      const INDEXER_SUBNETS = subnetsStr.includes(",")
        ? [...subnetsStr.split(",")]
        : [subnetsStr];

      const INDEXER_SECURITY_GROUP = process.env.INDEXER_SECURITY_GROUP ?? "";

      try {
        for (let i = 0; i < input.numberOfTasks; i++) {
          const subFrom = input.from + rangeSize * i;
          const subTo = Math.min(subFrom + rangeSize - 1, input.to);

          const commandOutput = await runTask(client, {
            cluster: INDEXER_ECS_CLUSTER,
            network: input.network,
            from: subFrom,
            to: subTo,
            subnets: INDEXER_SUBNETS,
            taskDefinition: INDEXER_TASK_DEF,
            logLevel: input.logLevel ?? "info",
            forceMode: input.forceMode ?? false,
            securityGroups: [INDEXER_SECURITY_GROUP],
          });

          for (const task of commandOutput.tasks ?? []) {
            if (task.taskArn) {
              const taskId = task.taskArn.split("/").pop();
              const creationDate = Math.floor(Date.now() / 1000);

              const putRequest = await dynamodb.putItem({
                TableName: tableName,
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
