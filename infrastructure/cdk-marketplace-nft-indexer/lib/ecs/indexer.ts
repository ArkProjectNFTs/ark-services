import * as cdk from "aws-cdk-lib";
import { IVpc } from "aws-cdk-lib/aws-ec2";
import { Cluster } from "aws-cdk-lib/aws-ecs";
import { LogGroup } from "aws-cdk-lib/aws-logs";
import { PolicyStatement } from "aws-cdk-lib/aws-iam";

export async function deployMarketplaceNftIndexer(
  scope: cdk.Stack,
  networks: string[],
  vpc: IVpc,
  environmentName: string,
  indexerVersion?: string
) {
  const cluster = new Cluster(scope, `marketplace-${environmentName}`, {
    vpc: vpc,
  });

  const ecrRepository = cdk.aws_ecr.Repository.fromRepositoryName(
    scope,
    "ArkProjectRepository",
    "ark-project-repo"
  );

  networks.forEach((network) => {
    deploy(
      environmentName,
      scope,
      cluster,
      ecrRepository,
      network,
      indexerVersion
    );
  });
}

function deploy(
  environmentName: string,
  scope: cdk.Stack,
  cluster: cdk.aws_ecs.ICluster,
  ecrRepository: cdk.aws_ecr.IRepository,
  network: string,
  indexerVersion?: string
) {
  const logGroup = new LogGroup(
    scope,
    `/ecs/marketplace-nft-indexer-${environmentName}-${network}`,
    {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      retention: 7,
    }
  );

  const taskDefinition = new cdk.aws_ecs.FargateTaskDefinition(
    scope,
    `marketplace-nft-indexer-${environmentName}-${network}`,
    {
      memoryLimitMiB: 2048,
      cpu: 512,
    }
  );

  const rpcProviderUri = network.includes("mainnet")
    ? `https://juno.mainnet.arkproject.dev`
    : `https://sepolia.arkproject.dev`;

  const environment: { [key: string]: string } = {
    CHAIN_ID: "0x534e5f4d41494e",
    HEAD_OF_CHAIN: "true",
    IPFS_GATEWAY_URI: "https://ipfs.arkproject.dev/ipfs/",
    RPC_PROVIDER: rpcProviderUri,
    RUST_LOG: "INFO",
    AWS_SECRET_NAME: "prod/ark-db-credentials",
    AWS_NFT_IMAGE_BUCKET_NAME: "ark-nft-media-mainnet",
  };

  if (indexerVersion) {
    environment.INDEXER_VERSION = indexerVersion;
  }

  taskDefinition.addContainer("indexer-marketplace", {
    image: cdk.aws_ecs.ContainerImage.fromEcrRepository(
      ecrRepository,
      `indexer-marketplace-${environmentName}-latest`
    ),
    logging: cdk.aws_ecs.LogDrivers.awsLogs({
      streamPrefix: "ecs",
      logGroup: logGroup,
    }),
    environment,
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

  new cdk.aws_ecs.FargateService(
    scope,
    `nft-indexer-${network.toLowerCase()}`,
    {
      cluster: cluster,
      taskDefinition: taskDefinition,
      desiredCount: 0,
    }
  );
}
