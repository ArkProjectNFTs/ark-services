import { DynamoDB } from "@aws-sdk/client-dynamodb";

import { type Network } from "~/types";
import { numbersNotInRange } from "./range";

const AWS_REGION = "us-east-1";

const dynamodb = new DynamoDB({ region: AWS_REGION });

export async function fetchBlocks(network: Network, latest: number) {
  const existingBlocks: number[] = [];
  const dynamoResult = await dynamodb.query({
    TableName: `ark_project_${network}`,
    IndexName: "GSI1PK-GSI1SK-index",
    KeyConditionExpression: "#GSI1PK = :GSI1PK",
    ExpressionAttributeNames: { "#GSI1PK": "GSI1PK" },
    ExpressionAttributeValues: { ":GSI1PK": { S: "BLOCK" } },
  });

  dynamoResult.Items?.forEach((item) => {
    const blockString = item.PK?.S?.split("#")[1];

    if (blockString && item.Type?.M?.Status?.S !== "None") {
      existingBlocks.push(+blockString);
    }
  });

  return numbersNotInRange(existingBlocks, 1, latest);
}
