import { contractRouter } from "~/server/api/routers/contract";
import { indexerRouter } from "~/server/api/routers/indexer";
import { mediaRouter } from "~/server/api/routers/media";
import { createTRPCRouter } from "~/server/api/trpc";

/**
 * This is the primary router for your server.
 *
 * All routers added in /api/routers should be manually added here.
 */
export const appRouter = createTRPCRouter({
  indexer: indexerRouter,
  contract: contractRouter,
  media: mediaRouter,
});

// export type definition of API
export type AppRouter = typeof appRouter;
