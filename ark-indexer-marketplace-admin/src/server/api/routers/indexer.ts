import { z } from "zod";

import { env } from "~/env.mjs";
import { spawnNFTIndexerTask } from "~/lib/aws";
import {
  fetchBlocks,
  fetchIndexers,
  fetchLatestBlocks,
  insertIndexer,
} from "~/lib/queries/indexer";
import { fetchLastBlock } from "~/lib/starknet";
import { type Network } from "~/types";
import { createTRPCRouter, protectedProcedure } from "../trpc";

type IndexerTask = {
  indexer_identifier: string;
  indexer_status: string;
  last_updated_timestamp: number;
  created_timestamp: number;
  indexer_version: string;
  indexation_progress_percentage: number;
  current_block_number: number;
  is_force_mode_enabled: boolean;
  start_block_number: number;
  end_block_number: number;
};

const fetchTasks = async (): Promise<IndexerTask[]> => {
  const indexers = await fetchIndexers();

  // eslint-disable-next-line @typescript-eslint/no-unsafe-return
  return indexers;
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
      const totalCount = 0;
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
    .query(async () => {
      try {
        const tasks = await fetchTasks();
        return tasks.map((t) => ({
          indexationProgress: t.indexation_progress_percentage,
          taskId: t.indexer_identifier,
          from: t.start_block_number,
          to: t.end_block_number,
          version: t.indexer_version,
          updatedAt: t.last_updated_timestamp,
          createdAt: t.created_timestamp,
        }));
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

      const INDEXER_ECS_CLUSTER = env.ARN_ECS_INDEXER_CLUSTER ?? "";
      const INDEXER_TASK_DEF = env.INDEXER_TASK_DEFINITION ?? "";
      const subnetsStr = env.INDEXER_SUBNETS ?? "";
      const INDEXER_SUBNETS = subnetsStr.includes(",")
        ? [...subnetsStr.split(",")]
        : [subnetsStr];

      const INDEXER_SECURITY_GROUP = process.env.INDEXER_SECURITY_GROUP ?? "";

      try {
        for (let i = 0; i < input.numberOfTasks; i++) {
          const subFrom = input.from + rangeSize * i;
          const subTo = Math.min(subFrom + rangeSize - 1, input.to);

          const commandOutput = await spawnNFTIndexerTask({
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
              if (taskId) {
                await insertIndexer(
                  taskId,
                  "LATEST",
                  0,
                  subFrom,
                  input.forceMode ?? false,
                  subFrom,
                  subTo,
                );
              }
            }
          }
        }
      } catch (error) {
        console.error(error);
      }
    }),
});
