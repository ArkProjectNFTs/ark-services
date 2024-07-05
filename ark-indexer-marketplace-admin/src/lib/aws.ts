/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import {
  ECSClient,
  RunTaskCommand,
  type RunTaskCommandInput,
} from "@aws-sdk/client-ecs";
import { PutObjectCommand, S3Client } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

interface NFTIndexerTaskOptions {
  cluster: string;
  network:
    | "production-sepolia"
    | "production-mainnet"
    | "staging-mainnet"
    | "staging-sepolia";
  taskDefinition: string;
  subnets: string[];
  from: number;
  to: number;
  logLevel: string;
  forceMode: boolean;
  securityGroups: string[];
}

interface MetadataIndexerTaskOptions {
  contractAddress: string;
  cluster: string;
  taskDefinition: string;
  subnets: string[];
  securityGroups: string[];
}

export const client = new ECSClient({
  region: "us-east-1",
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID!,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY!,
  },
});

export async function spawnMetadataIndexerTask(
  options: MetadataIndexerTaskOptions,
) {
  const input: RunTaskCommandInput = {
    cluster: options.cluster,
    taskDefinition: options.taskDefinition,
    launchType: "FARGATE",
    networkConfiguration: {
      awsvpcConfiguration: {
        subnets: options.subnets,
        securityGroups: options.securityGroups,
        assignPublicIp: "ENABLED",
      },
    },
    overrides: {
      containerOverrides: [
        {
          name: "indexer-metadata-marketplace",
          environment: [
            {
              name: "RPC_PROVIDER",
              value: "https://juno.mainnet.arkproject.dev",
            },
            {
              name: "AWS_NFT_IMAGE_BUCKET_NAME",
              value: "ark-nft-images-mainnet",
            },
            {
              name: "AWS_SECRET_NAME",
              value: "prod/ark-db-credentials",
            },
            {
              name: "METADATA_IPFS_TIMEOUT_IN_SEC",
              value: "5",
            },
            {
              name: "METADATA_LOOP_DELAY_IN_SEC",
              value: "10",
            },
            {
              name: "IPFS_GATEWAY_URI",
              value: "https://ipfs.arkproject.dev/ipfs/",
            },
            {
              name: "RUST_LOG",
              value: "INFO",
            },
            {
              name: "REFRESH_CONTRACT_METADATA",
              value: "true",
            },
            {
              name: "METADATA_CONTRACT_FILTER",
              value: options.contractAddress,
            },
            {
              name: "CHAIN_ID_FILTER",
              value: "0x534e5f4d41494e",
            },
          ],
        },
      ],
    },
  };

  const command = new RunTaskCommand(input);
  const output = await client.send(command);
  return output;
}

export async function spawnNFTIndexerTask(options: NFTIndexerTaskOptions) {
  const input: RunTaskCommandInput = {
    cluster: options.cluster,
    taskDefinition: options.taskDefinition,
    launchType: "FARGATE",
    networkConfiguration: {
      awsvpcConfiguration: {
        subnets: options.subnets,
        securityGroups: options.securityGroups,
        assignPublicIp: "ENABLED",
      },
    },
    overrides: {
      containerOverrides: [
        {
          name: "indexer-marketplace",
          environment: [
            {
              name: "RPC_PROVIDER",
              value: options.network.includes("mainnet")
                ? `https://juno.mainnet.arkproject.dev`
                : `https://sepolia.arkproject.dev`,
            },
            {
              name: "HEAD_OF_CHAIN",
              value: "false",
            },
            {
              name: "FROM_BLOCK",
              value: `${options.from}`,
            },
            {
              name: "TO_BLOCK",
              value: `${options.to}`,
            },
            {
              name: "RUST_LOG",
              value: options.logLevel,
            },
            {
              name: "FORCE_MODE",
              value: `${options.forceMode}`,
            },
          ],
        },
      ],
    },
  };

  const command = new RunTaskCommand(input);
  const output = await client.send(command);
  return output;
}

export async function createPresignedUrl(key: string): Promise<string> {
  const client = new S3Client({ region: "us-east-1" });
  const command = new PutObjectCommand({
    Bucket: "ark-nft-media-mainnet",
    Key: key,
  });

  const signedUrl = await getSignedUrl(client, command, { expiresIn: 3600 });
  return signedUrl;
}
