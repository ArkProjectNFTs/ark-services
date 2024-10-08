import { z } from "zod";

import { env } from "~/env.mjs";
import { spawnMetadataIndexerTask } from "~/lib/aws";
import {
  fetchContract,
  fetchRefreshingContracts,
  getDefaultContracts,
  searchContracts,
  updateContract,
  updateIsRefreshingContract,
} from "~/lib/queries/contract";
import { clearListedTokensCache } from "~/lib/redis";
import { createTRPCRouter, protectedProcedure } from "../trpc";

const MAINNET_CHAIN_ID = "0x534e5f4d41494e"; // Hardcoded value

export const contractRouter = createTRPCRouter({
  flushCache: protectedProcedure
    .input(
      z.object({
        contractAddress: z.string(),
      }),
    )
    .mutation(async ({ input }) => {
      await clearListedTokensCache(input.contractAddress);
    }),

  getContracts: protectedProcedure.query(async () => {
    try {
      const contracts = await getDefaultContracts(MAINNET_CHAIN_ID);

      console.log("Contracts found:", JSON.stringify(contracts, null, 2));
      return contracts;
    } catch (err) {
      return [];
    }
  }),

  searchContracts: protectedProcedure
    .input(
      z.object({
        contractName: z.string(),
      }),
    )
    .query(async ({ input }: { input: { contractName: string } }) => {
      console.log("Searching contracts with name:", input.contractName);

      try {
        const contracts = await searchContracts(
          input.contractName,
          MAINNET_CHAIN_ID,
        );

        console.log("Contracts found:", JSON.stringify(contracts, null, 2));
        return contracts;
      } catch (err) {
        return [];
      }
    }),

  getRefreshingContracts: protectedProcedure
    .input(z.object({}))
    .query(async () => {
      const contracts = await fetchRefreshingContracts(MAINNET_CHAIN_ID);

      return contracts.map((contract) => {
        const tokenCount = contract.token_count || 0;
        const refreshedTokenCount = contract.refreshed_token_count || 0;
        const progression =
          tokenCount > 0
            ? parseFloat(((refreshedTokenCount / tokenCount) * 100).toFixed(2))
            : 0;

        return {
          ...contract,
          progression,
        };
      });
    }),

  getContract: protectedProcedure
    .input(
      z.object({
        contractAddress: z.string(),
      }),
    )
    .query(async ({ input }: { input: { contractAddress: string } }) => {
      const contract = await fetchContract(
        input.contractAddress,
        MAINNET_CHAIN_ID,
      );

      return contract;
    }),

  refreshContractMetadata: protectedProcedure
    .input(
      z.object({
        contractAddress: z.string(),
      }),
    )
    .mutation(async ({ input }) => {
      const contract = await fetchContract(
        input.contractAddress,
        MAINNET_CHAIN_ID,
      );

      if (contract?.is_refreshing) {
        throw new Error("Contract is already refreshing");
      }

      await updateIsRefreshingContract(
        input.contractAddress,
        MAINNET_CHAIN_ID,
        true,
      );

      const subnets = env.MARKETPLACE_INDEXER_SUBNETS.includes(",")
        ? [...env.MARKETPLACE_INDEXER_SUBNETS.split(",")]
        : [env.MARKETPLACE_INDEXER_SUBNETS];

      const securityGroups = env.MARKETPLACE_INDEXER_SECURITY_GROUPS.includes(
        ",",
      )
        ? [...env.MARKETPLACE_INDEXER_SECURITY_GROUPS.split(",")]
        : [env.MARKETPLACE_INDEXER_SECURITY_GROUPS];

      await spawnMetadataIndexerTask({
        cluster: env.MARKETPLACE_INDEXER_CLUSTER,
        securityGroups,
        subnets,
        taskDefinition: env.MARKETPLACE_INDEXER_TASK_DEFINITION,
        contractAddress: input.contractAddress,
      });
    }),

  updateContract: protectedProcedure
    .input(
      z.object({
        name: z.string(),
        image: z.string().optional(),
        isSpam: z.boolean(),
        isNSFW: z.boolean(),
        isVerified: z.boolean(),
        saveImages: z.boolean(),
        symbol: z.string(),
        contractAddress: z.string(),
      }),
    )
    .mutation(async ({ input }) => {
      await updateContract(
        input.name,
        input.symbol,
        input.isSpam,
        input.isNSFW,
        input.isVerified,
        input.saveImages,
        input.contractAddress,
        MAINNET_CHAIN_ID,
        input.image,
      );

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
});
