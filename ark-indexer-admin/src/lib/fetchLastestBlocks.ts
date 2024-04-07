import type {
  AttributeValue,
  QueryCommandOutput,
} from "@aws-sdk/client-dynamodb";

import { db } from "~/server/dynamodb";
import type { Network } from "~/types";
import { getTableName } from "./utils";

export async function fetchLatestBlocks(
  network: Network,
): Promise<Record<string, AttributeValue>[]> {
  try {
    const result: QueryCommandOutput = await db.query({
      TableName: getTableName(network),
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
  } catch (err) {
    console.error("Error fetching latest blocks: ", err);
    return [];
  }
}
