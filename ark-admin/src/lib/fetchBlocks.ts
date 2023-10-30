import { db } from "~/server/dynamodb";
import { type Network } from "~/types";
import { type Range } from "./range";

export async function fetchBlocks(network: Network, latest: number) {
  const dynamoResult = await db.query({
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
  });

  const count = latest - (dynamoResult.Items?.length ?? 0);
  const rangeCount = 120;
  const rangeSize = Math.ceil(latest / rangeCount);
  const ranges: Range[] = [];

  // Initialize the ranges with empty blocks arrays
  for (let i = 0; i < rangeCount; i++) {
    const start = i * rangeSize;
    const end = i !== rangeCount - 1 ? (i + 1) * rangeSize - 1 : latest;
    ranges.push({ start, end, blocks: [] });
  }

  let nextExpectedBlock = 0;

  for (const item of dynamoResult.Items ?? []) {
    const block = +(item.PK?.S?.split("#")[1] ?? 0);

    // Fill in missing blocks until the current block
    while (nextExpectedBlock < block) {
      const rangeIndex = Math.floor(nextExpectedBlock / rangeSize);
      ranges[rangeIndex]?.blocks.push(nextExpectedBlock);
      nextExpectedBlock++;
    }

    // Set the next expected block
    nextExpectedBlock = block + 1;
  }

  // Fill in missing blocks for the remainder of the range
  while (nextExpectedBlock <= latest) {
    const rangeIndex = Math.floor(nextExpectedBlock / rangeSize);
    ranges[rangeIndex]?.blocks.push(nextExpectedBlock);
    nextExpectedBlock++;
  }

  return { ranges, rangeSize, count };
}
