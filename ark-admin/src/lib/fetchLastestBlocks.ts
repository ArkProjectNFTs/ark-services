import type {
  AttributeValue,
  QueryCommandOutput,
} from "@aws-sdk/client-dynamodb";

import { db } from "~/server/dynamodb";

type NetworkType = "mainnet" | "testnet";

export async function fetchLatestBlocks(
  network: NetworkType,
): Promise<Record<string, AttributeValue>[]> {
  const result: QueryCommandOutput = await db.query({
    TableName: `${process.env.TABLE_NAME_PREFIX}${network}`,
    IndexName: "GSI1PK-GSI1SK-index",
    KeyConditionExpression: "#GSI1PK = :GSI1PK",
    ExpressionAttributeNames: {
      "#GSI1PK": "GSI1PK",
    },
    ExpressionAttributeValues: {
      ":GSI1PK": { S: "BLOCK" },
    },
    Limit: 10,
    ScanIndexForward: false,
  });

  return result.Items ?? [];
}
