import { DynamoDB } from "@aws-sdk/client-dynamodb";

import { env } from "~/env.mjs";

const globalForDynamoDB = globalThis as unknown as {
  dynamodb: DynamoDB | undefined;
};

export const db = globalForDynamoDB.dynamodb ?? new DynamoDB();

if (env.NODE_ENV !== "production") {
  globalForDynamoDB.dynamodb = db;
}
