import { z } from "zod";

import { createPresignedUrl } from "~/lib/aws";
import { createTRPCRouter, protectedProcedure } from "../trpc";

export const mediaRouter = createTRPCRouter({
  generateMediaPresignedUri: protectedProcedure
    .input(
      z.object({
        mediaKey: z.string(),
      }),
    )
    .query(async ({ input }: { input: { mediaKey: string } }) => {
      const avatarPresignedUrl = await createPresignedUrl(input.mediaKey);
      return avatarPresignedUrl;
    }),
});
