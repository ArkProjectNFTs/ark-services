import { createEnv } from "@t3-oss/env-nextjs";
import { z } from "zod";

export const env = createEnv({
  shared: {
    VERCEL_URL: z
      .string()
      .optional()
      .transform((v) => (v ? `https://${v}` : undefined)),
    PORT: z.coerce.number().default(3000),
  },
  server: {
    REDIS_URL: z.string(),
    PGUSER: z.string(),
    PGPASSWORD: z.string(),
    PGHOST: z.string(),
    PGPORT: z.string(),
    PGDATABASE: z.string(),
    RESEND_API_KEY: z.string().min(1),
    STARKNET_TESTNET_RPC_PROVIDER: z.string().min(1),
    STARKNET_MAINNET_RPC_PROVIDER: z.string().min(1),
    NODE_ENV: z
      .enum(["development", "test", "production"])
      .default("development"),
  },
  client: {},
  runtimeEnv: {
    REDIS_URL: process.env.REDIS_URL,
    NODE_ENV: process.env.NODE_ENV,
    VERCEL_URL: process.env.VERCEL_URL,
    PORT: process.env.PORT ?? "3000",
    RESEND_API_KEY: process.env.RESEND_API_KEY,
    STARKNET_TESTNET_RPC_PROVIDER: process.env.STARKNET_TESTNET_RPC_PROVIDER,
    STARKNET_MAINNET_RPC_PROVIDER: process.env.STARKNET_MAINNET_RPC_PROVIDER,
    PGUSER: process.env.PGUSER,
    PGPASSWORD: process.env.PGPASSWORD,
    PGHOST: process.env.PGHOST,
    PGPORT: process.env.PGPORT,
    PGDATABASE: process.env.PGDATABASE,
  },
  skipValidation:
    !!process.env.CI ||
    !!process.env.SKIP_ENV_VALIDATION ||
    process.env.npm_lifecycle_event === "lint",
});
