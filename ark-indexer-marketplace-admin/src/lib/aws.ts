/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import {
  RunTaskCommand,
  type ECSClient,
  type RunTaskCommandInput,
} from "@aws-sdk/client-ecs";
import { PutObjectCommand, S3Client } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

interface SpawnTaskOptions {
  cluster: string;
  network: string;
  taskDefinition: string;
  subnets: string[];
  from: number;
  to: number;
  logLevel: string;
  forceMode: boolean;
  securityGroups: string[];
}

export const runTask = async (client: ECSClient, options: SpawnTaskOptions) => {
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
          name: "ark_indexer",
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
};

export async function createPresignedUrl(key: string): Promise<string> {
  const client = new S3Client({ region: "us-east-1" });
  const command = new PutObjectCommand({
    Bucket: "ark-nft-media-mainnet",
    Key: key,
  });

  const signedUrl = await getSignedUrl(client, command, { expiresIn: 3600 });
  return signedUrl;
}
