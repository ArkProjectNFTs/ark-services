import type { AttributeValue } from "@aws-sdk/client-dynamodb";

import { db } from "~/server/dynamodb";
import { type Network } from "~/types";
import { type Range } from "./range";

/**
 * Fetches blocks from the database, calculates ranges and returns them.
 *
 * @param {Network} network - Network type parameter.
 * @param {number} latest - Latest block number.
 *
 * @returns {Object} - Returns ranges, rangeSize, and count.
 */
export async function fetchBlocks(network: Network, latest: number) {
  const allItems: Record<string, AttributeValue>[] =
    await fetchAllDynamoItems(network);
  const count = latest - allItems.length;
  const rangeCount = 120;
  const rangeSize = Math.ceil(latest / rangeCount);

  const ranges: Range[] = createEmptyRanges(latest, rangeCount, rangeSize);
  populateRangesWithBlocks(ranges, allItems, rangeSize, latest);

  return { ranges, rangeSize, count };
}

/**
 * Fetch all items from DynamoDB for a given network.
 *
 * @param {Network} network - Network type parameter.
 *
 * @returns {Promise<Record<string, AttributeValue>[]>} - Returns all fetched items.
 */
async function fetchAllDynamoItems(
  network: Network,
): Promise<Record<string, AttributeValue>[]> {
  let lastEvaluatedKey: Record<string, AttributeValue> | undefined = undefined;
  const allItems: Record<string, AttributeValue>[] = [];

  while (true) {
    const dynamoQueryResult = await db.query({
      TableName: `ark_project_${network}`,
      IndexName: "GSI1PK-GSI1SK-index",
      KeyConditionExpression: "#GSI1PK = :GSI1PK",
      ExpressionAttributeNames: {
        "#GSI1PK": "GSI1PK",
        "#status": "Data.M.Status",
      },
      ExpressionAttributeValues: {
        ":GSI1PK": { S: "BLOCK" },
        ":noneValue": { S: "NONE" },
      },
      FilterExpression: "#status <> :noneValue",
      ProjectionExpression: "PK",
      ExclusiveStartKey: lastEvaluatedKey,
    });

    allItems.push(...(dynamoQueryResult.Items ?? []));

    if (dynamoQueryResult.LastEvaluatedKey) {
      const tmp: Record<string, AttributeValue> =
        dynamoQueryResult.LastEvaluatedKey;
      lastEvaluatedKey = tmp;
    } else {
      break;
    }
  }

  return allItems;
}

/**
 * Creates a list of empty ranges based on provided parameters.
 *
 * @param {number} latest - Latest block number.
 * @param {number} rangeCount - Total count of ranges.
 * @param {number} rangeSize - Size of each range.
 *
 * @returns {Range[]} - Returns an array of empty ranges.
 */
function createEmptyRanges(
  latest: number,
  rangeCount: number,
  rangeSize: number,
): Range[] {
  return Array.from({ length: rangeCount }, (_, i) => {
    const start = i * rangeSize;
    const end = i !== rangeCount - 1 ? (i + 1) * rangeSize - 1 : latest;
    return { start, end, blocks: [] };
  });
}

/**
 * Populates the ranges with blocks from the list of all items.
 *
 * @param {Range[]} ranges - Array of ranges.
 * @param {Record<string, AttributeValue>[]} allItems - All fetched items.
 * @param {number} rangeSize - Size of each range.
 * @param {number} latest - Latest block number.
 */
function populateRangesWithBlocks(
  ranges: Range[],
  allItems: Record<string, AttributeValue>[],
  rangeSize: number,
  latest: number,
) {
  let nextExpectedBlock = 0;

  for (const item of allItems) {
    const block = +(item.PK?.S?.split("#")[1] ?? 0);

    while (nextExpectedBlock < block) {
      const rangeIndex = Math.floor(nextExpectedBlock / rangeSize);
      ranges[rangeIndex]?.blocks.push(nextExpectedBlock);
      nextExpectedBlock++;
    }

    nextExpectedBlock = block + 1;
  }

  while (nextExpectedBlock <= latest) {
    const rangeIndex = Math.floor(nextExpectedBlock / rangeSize);
    ranges[rangeIndex]?.blocks.push(nextExpectedBlock);
    nextExpectedBlock++;
  }
}
