import {
  DynamoDB,
  type AttributeValue,
  QueryCommandOutput,
} from "@aws-sdk/client-dynamodb";
import { config } from "dotenv";
import { promises as fs } from "fs";
import { RpcProvider, Contract, shortString, num } from "starknet";

config();

const tableName = "ark_project_staging_mainnet";

async function saveAllContracts(dynamodb: DynamoDB) {
  const contracts: string[] = [];
  let lastEvaluatedKey: Record<string, AttributeValue> | undefined = undefined;
  do {
    const dynamoResult: QueryCommandOutput = await dynamodb.query({
      TableName: tableName,
      IndexName: "GSI2PK-GSI2SK-index",
      KeyConditionExpression: "#GSI2PK = :GSI2PK",
      ExpressionAttributeNames: { "#GSI2PK": "GSI2PK" },
      ExpressionAttributeValues: {
        ":GSI2PK": { S: `NFT` },
      },
      ExclusiveStartKey: lastEvaluatedKey,
    });

    if (dynamoResult) {
      if (dynamoResult.Items) {
        for (const item of dynamoResult.Items) {
          const contractAddress = item.Data.M?.ContractAddress.S;
          if (contractAddress && !contracts.includes(contractAddress)) {
            console.log(contractAddress);
            contracts.push(contractAddress);
          }

          //   const timestamp = item.Data?.M?.Timestamp?.N;

          //   // Update the item with new SK
          //   const updateParams = {
          //     TableName: tableName,
          //     Key: {
          //       PK: item.PK,
          //       SK: item.SK,
          //     },
          //     UpdateExpression: "SET GSI1SK = :GSI1SK",
          //     ExpressionAttributeValues: {
          //       ":GSI1SK": { S: `EVENT#${timestamp}` },
          //     },
          //   };

          //   try {
          //     await dynamodb.updateItem(updateParams);
          //     console.log(`Updated item: ${JSON.stringify(item)}`);
          //   } catch (error) {
          //     console.error(`Error updating item: ${JSON.stringify(error)}`);
          //   }
        }
      }

      lastEvaluatedKey = dynamoResult.LastEvaluatedKey;
    }
  } while (lastEvaluatedKey);

  // Save contracts to file

  await fs.writeFile("existing-contracts.json", JSON.stringify(contracts));
}

async function getContractMetadata(dynamodb: DynamoDB) {
  await saveAllContracts(dynamodb);

  const results = await fs.readFile("existing-contracts.json", "utf-8");
  const contracts = JSON.parse(results);

  const provider = new RpcProvider({
    nodeUrl: "https://starknet-mainnet.public.blastapi.io",
  });

  let existingContracts: Record<string, { name: string; symbol: string }> = {};

  for (let i = 0; i < contracts.length; i += 1) {
    const contractAddress = contracts[i];
    console.log(i);
    const contract = new Contract(
      [
        {
          name: "name",
          type: "function",
          inputs: [],
          outputs: [
            {
              name: "name",
              type: "felt",
            },
          ],
          stateMutability: "view",
        },
        {
          name: "symbol",
          type: "function",
          inputs: [],
          outputs: [
            {
              name: "symbol",
              type: "felt",
            },
          ],
          stateMutability: "view",
        },
      ],
      contractAddress,
      provider
    );

    try {
      const nameResult = await contract.name();
      const name = shortString.decodeShortString(nameResult.name);

      const symbolResult = await contract.symbol();
      const symbol = shortString.decodeShortString(symbolResult.symbol);

      console.log("name", name);
      console.log("symbol", symbol);

      existingContracts = {
        ...existingContracts,
        [contractAddress]: { name, symbol },
      };
    } catch (err) {
      console.error("Error with contract", contractAddress);
    }
  }

  await fs.writeFile(
    "existing-contracts-metadata.json",
    JSON.stringify(existingContracts)
  );

  console.log("Done !!!");
}

// (async () => {
//   const dynamodb = new DynamoDB({ region: "us-east-1" });
//   await getContractMetadata(dynamodb);
// })();

(async () => {
  const dynamodb = new DynamoDB({ region: "us-east-1" });

  const results = await fs.readFile(
    "existing-contracts-metadata.json",
    "utf-8"
  );
  const contracts = JSON.parse(results);
  console.log(contracts);

  for (const contractAddress of Object.keys(contracts)) {
    const { name, symbol } = contracts[contractAddress];

    try {
      await dynamodb.updateItem({
        TableName: tableName,
        Key: {
          PK: { S: `CONTRACT#${contractAddress}` },
          SK: { S: `CONTRACT` },
        },
        UpdateExpression: "SET #Data.#Name = :Name, #Data.#Symbol = :Symbol",
        ExpressionAttributeNames: {
          "#Data": "Data",
          "#Name": "Name",
          "#Symbol": "Symbol",
        },
        ExpressionAttributeValues: {
          ":Name": { S: name },
          ":Symbol": { S: symbol },
        },
      });

      console.log("Updated item:", `CONTRACT#${contractAddress}`);
    } catch (error) {
      console.error(`Error updating item: ${JSON.stringify(error)}`);
    }
  }
})();
