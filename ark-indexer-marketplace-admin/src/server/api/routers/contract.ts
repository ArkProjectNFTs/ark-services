import { z } from "zod";

import {
  fetchContract,
  fetchRefreshingContracts,
  searchContracts,
  updateContract,
} from "~/lib/queries/contract";
import { createTRPCRouter, protectedProcedure } from "../trpc";

export const contractRouter = createTRPCRouter({
  searchContracts: protectedProcedure
    .input(
      z.object({
        contractName: z.string(),
      }),
    )
    .query(async ({ input }: { input: { contractName: string } }) => {
      console.log("Searching contracts with name:", input.contractName);

      const contracts = await searchContracts(
        input.contractName,
        "0x534e5f4d41494e",
      );

      return contracts;
    }),

  getRefreshingContracts: protectedProcedure
    .input(z.object({}))
    .query(async () => {
      const contracts = await fetchRefreshingContracts("0x534e5f4d41494e");

      return contracts;
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
        "0x534e5f4d41494e",
      );

      return contract;
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
        "0x534e5f4d41494e",
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
