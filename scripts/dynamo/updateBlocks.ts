import {
  DynamoDB,
  type AttributeValue,
  QueryCommandOutput,
} from "@aws-sdk/client-dynamodb";
import { config } from "dotenv";

config();

(async () => {
  const dynamodb = new DynamoDB({ region: "us-east-1" });

  let lastEvaluatedKey: Record<string, AttributeValue> | undefined = undefined;
  do {
    const dynamoResult: QueryCommandOutput = await dynamodb.query({
      TableName: "ark_project_mainnet",
      IndexName: "GSI1PK-GSI1SK-index",
      KeyConditionExpression: "#GSI1PK = :GSI1PK",
      ExpressionAttributeNames: {
        "#GSI1PK": "GSI1PK",
        "#GSI6SK": "GSI6SK",
      },
      ExpressionAttributeValues: {
        ":GSI1PK": { S: `BLOCK` },
        ":empty": { S: "" },
      },
      FilterExpression: "attribute_not_exists(#GSI6SK) OR #GSI6SK = :empty",

      ScanIndexForward: false,
      ExclusiveStartKey: lastEvaluatedKey,
    });

    if (dynamoResult) {
      if (dynamoResult.Items) {
        for (const item of dynamoResult.Items) {
          const blockNumber = item.Data?.M?.BlockNumber?.N;
          console.log("=> blockNumber", blockNumber);
          if (blockNumber && item.GSI6PK?.S !== "BLOCK") {
            try {
              await dynamodb.updateItem({
                TableName: "ark_project_mainnet",
                Key: {
                  PK: item.PK,
                  SK: item.SK,
                },
                UpdateExpression: "SET GSI6PK = :GSI6PK, GSI6SK = :GSI6SK",
                ExpressionAttributeValues: {
                  ":GSI6PK": { S: `BLOCK` },
                  ":GSI6SK": { N: blockNumber },
                },
              });
              console.log(`Updated item: ${JSON.stringify(item)}`);
            } catch (error) {
              console.error(`Error updating item: ${JSON.stringify(error)}`);
            }
          }
        }
      }

      lastEvaluatedKey = dynamoResult.LastEvaluatedKey;
    }
  } while (lastEvaluatedKey);
})();
