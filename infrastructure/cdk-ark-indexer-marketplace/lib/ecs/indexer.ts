import * as cdk from "aws-cdk-lib";
import { IVpc } from "aws-cdk-lib/aws-ec2";
import { Cluster } from "aws-cdk-lib/aws-ecs";
import { Table } from "aws-cdk-lib/aws-dynamodb";
import { LogGroup } from "aws-cdk-lib/aws-logs";
import { PolicyStatement } from "aws-cdk-lib/aws-iam";
import * as lambda from "aws-cdk-lib/aws-lambda";
import * as ssm from "aws-cdk-lib/aws-ssm";

export async function deployIndexers(
  scope: cdk.Stack,
  networks: string[],
  vpc: IVpc,
  isProductionEnvironment: boolean,
  environmentName: string,
  indexerVersion: string
) {
  const cluster = new Cluster(scope, "arkproject", {
    vpc: vpc,
  });

  const ecrRepository = cdk.aws_ecr.Repository.fromRepositoryName(
    scope,
    "ArkProjectRepository",
    "ark-project-repo"
  );

  networks.forEach((network) => {
    deployIndexerServices(
      isProductionEnvironment,
      environmentName,
      scope,
      cluster,
      ecrRepository,
      network,
      indexerVersion
    );
  });
}

function deployIndexerServices(
  isProductionEnvironment: boolean,
  environmentName: string,
  scope: cdk.Stack,
  cluster: cdk.aws_ecs.ICluster,
  ecrRepository: cdk.aws_ecr.IRepository,
  network: string,
  indexerVersion: string
) {
  const logGroup = new LogGroup(
    scope,
    `/ecs/indexer-marketplace-${environmentName}-${network}`,
    {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      retention: 7,
    }
  );

  const taskDefinition = new cdk.aws_ecs.FargateTaskDefinition(
    scope,
    `indexer-marketplace-${environmentName}-${network}-task-definition`,
    {
      memoryLimitMiB: 2048,
      cpu: 512,
    }
  );

  const rpcProviderUri = network.includes("mainnet")
    ? `https://juno.mainnet.arkproject.dev`
    : `https://sepolia.arkproject.dev`;

  taskDefinition.addContainer("indexer-marketplace", {
    image: cdk.aws_ecs.ContainerImage.fromEcrRepository(
      ecrRepository,
      `indexer-marketplace-${environmentName}-latest`
    ),
    logging: cdk.aws_ecs.LogDrivers.awsLogs({
      streamPrefix: "ecs",
      logGroup: logGroup,
    }),
    environment: {
      CHAIN_ID: "0x534e5f4d41494e",
      HEAD_OF_CHAIN: "true",
      INDEXER_VERSION: indexerVersion,
      IPFS_GATEWAY_URI: "https://ipfs.arkproject.dev/ipfs/",
      RPC_PROVIDER: rpcProviderUri,
      RUST_LOG: "INFO",
      AWS_SECRET_NAME: "prod/ark-db-credentials",
      AWS_NFT_IMAGE_BUCKET_NAME: "ark-nft-media-mainnet",
    },
  });

  taskDefinition.addToTaskRolePolicy(
    new PolicyStatement({
      actions: ["s3:PutObject"],
      resources: ["arn:aws:s3:::ark-nft-media-mainnet/*"],
    })
  );

  taskDefinition.addToTaskRolePolicy(
    new PolicyStatement({
      actions: ["logs:CreateLogStream", "logs:PutLogEvents"],
      resources: ["*"],
    })
  );

  taskDefinition.addToTaskRolePolicy(
    new PolicyStatement({
      actions: ["secretsmanager:GetSecretValue"],
      resources: ["*"],
    })
  );

  const capitalizedNetwork =
    network.charAt(0).toUpperCase() + network.slice(1).toLowerCase();

  new cdk.aws_ecs.FargateService(scope, `HeadOfChain${capitalizedNetwork}`, {
    cluster: cluster,
    taskDefinition: taskDefinition,
    desiredCount: isProductionEnvironment ? 1 : 0,
  });
}
