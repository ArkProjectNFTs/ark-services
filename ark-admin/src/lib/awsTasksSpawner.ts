import {
  RunTaskCommand,
  type ECSClient,
  type RunTaskCommandInput,
} from "@aws-sdk/client-ecs";

interface SpawnTaskOptions {
  cluster: string;
  network: string;
  taskDefinition: string;
  subnetId: string;
  from: number;
  to: number;
  logLevel: string;
  forceMode: boolean;
}

export const runTask = async (client: ECSClient, options: SpawnTaskOptions) => {
  const input: RunTaskCommandInput = {
    cluster: options.cluster,
    taskDefinition:
      options.network === "mainnet"
        ? "ark-indexer-task-mainnet"
        : "ark-indexer-task-testnet",
    launchType: "FARGATE",
    networkConfiguration: {
      awsvpcConfiguration: {
        subnets: [options.subnetId],
        securityGroups: ["sg-0c1b5d08ea088eb53"],
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
              value: `https://juno.${
                options.network === "mainnet" ? "mainnet" : "testnet"
              }.arkproject.dev`,
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
