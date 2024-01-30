import {
  DynamoDB,
  type AttributeValue,
  QueryCommandOutput,
} from "@aws-sdk/client-dynamodb";
import { config } from "dotenv";

config();

(async () => {
  const dynamodb = new DynamoDB({ region: "us-east-1" });

  const contractAddress =
    "0x076503062d78f4481be03c9145022d6a4a71ec0719aa07756f79a2384dc7ef16";

  let lastEvaluatedKey: Record<string, AttributeValue> | undefined = undefined;
  do {
    const dynamoResult: QueryCommandOutput = await dynamodb.query({
      TableName: "ark_project_mainnet",
      IndexName: "GSI1PK-GSI1SK-index",
      KeyConditionExpression:
        "#GSI1PK = :GSI1PK AND begins_with(GSI1SK, :event)",
      ExpressionAttributeNames: { "#GSI1PK": "GSI1PK" },
      ExpressionAttributeValues: {
        ":GSI1PK": { S: `CONTRACT#${contractAddress}` },
        ":event": { S: "EVENT#0x" },
      },
      ScanIndexForward: false,
      ExclusiveStartKey: lastEvaluatedKey,
    });

    if (dynamoResult) {
      if (dynamoResult.Items) {
        for (const item of dynamoResult.Items) {
          console.log("=> item", item.PK.S);

          const timestamp = item.Data?.M?.Timestamp?.N;

          // Update the item with new SK
          const updateParams = {
            TableName: "ark_project_mainnet",
            Key: {
              PK: item.PK,
              SK: item.SK,
            },
            UpdateExpression: "SET GSI1SK = :GSI1SK",
            ExpressionAttributeValues: {
              ":GSI1SK": { S: `EVENT#${timestamp}` },
            },
          };

          try {
            await dynamodb.updateItem(updateParams);
            console.log(`Updated item: ${JSON.stringify(item)}`);
          } catch (error) {
            console.error(`Error updating item: ${JSON.stringify(error)}`);
          }
        }
      }

      lastEvaluatedKey = dynamoResult.LastEvaluatedKey;
    }
  } while (lastEvaluatedKey);
})();
