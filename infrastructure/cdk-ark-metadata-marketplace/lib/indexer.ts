import * as cdk from "aws-cdk-lib";
import { IVpc } from "aws-cdk-lib/aws-ec2";
import { Cluster } from "aws-cdk-lib/aws-ecs";
import { LogGroup } from "aws-cdk-lib/aws-logs";
import { PolicyStatement } from "aws-cdk-lib/aws-iam";
import * as iam from "aws-cdk-lib/aws-iam";

export async function deployMetadataIndexer(
  scope: cdk.Stack,
  networks: string[],
  vpc: IVpc,
  isProductionEnvironment: boolean
) {
  const cluster = new Cluster(scope, "metadata-marketplace", {
    vpc: vpc,
  });

  const ecrRepository = cdk.aws_ecr.Repository.fromRepositoryName(
    scope,
    "ArkProjectRepository",
    "ark-project-repo"
  );

  networks.forEach((network) => {
    deployMetadataServices(
      scope,
      cluster,
      ecrRepository,
      network,
      isProductionEnvironment
    );
  });
}

function deployMetadataServices(
  scope: cdk.Stack,
  cluster: cdk.aws_ecs.ICluster,
  ecrRepository: cdk.aws_ecr.IRepository,
  network: string,
  isProductionEnvironment: boolean
) {
  const capitalizedNetwork =
    network.charAt(0).toUpperCase() + network.slice(1).toLowerCase();

  const logGroup = new LogGroup(
    scope,
    `/ecs/metadata-marketplace-${
      isProductionEnvironment ? "production" : "staging"
    }-${network}`,
    {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      retention: 7,
    }
  );

  const taskDefinition = new cdk.aws_ecs.FargateTaskDefinition(
    scope,
    `metadata-marketplace-${
      isProductionEnvironment ? "production" : "staging"
    }-${network}-task-definition`,
    {
      memoryLimitMiB: 2048,
      cpu: 512,
    }
  );

  if (isProductionEnvironment) {
    taskDefinition.taskRole.addToPrincipalPolicy(
      new iam.PolicyStatement({
        actions: ["secretsmanager:GetSecretValue"],
        resources: [
          "arn:aws:secretsmanager:us-east-1:223605539824:secret:prod/ark-db-credentials-dVvbBG",
        ],
      })
    );
  }

  const rpcProviderUri = network.includes("mainnet")
    ? `https://juno.mainnet.arkproject.dev`
    : `https://sepolia.arkproject.dev`;

  taskDefinition.addContainer(`indexer-metadata-marketplace`, {
    image: cdk.aws_ecs.ContainerImage.fromEcrRepository(
      ecrRepository,
      `metadata-marketplace-${
        isProductionEnvironment ? "production" : "staging"
      }-latest`
    ),
    logging: cdk.aws_ecs.LogDrivers.awsLogs({
      streamPrefix: "ecs",
      logGroup: logGroup,
    }),
    environment: {
      AWS_NFT_IMAGE_BUCKET_NAME: `ark-nft-images-${network}`,
      RPC_PROVIDER: rpcProviderUri,
      METADATA_IPFS_TIMEOUT_IN_SEC:
        process.env.METADATA_IPFS_TIMEOUT_IN_SEC ?? "5",
      METADATA_LOOP_DELAY_IN_SEC:
        process.env.METADATA_LOOP_DELAY_IN_SEC ?? "10",
      IPFS_GATEWAY_URI:
        process.env.IPFS_GATEWAY_URI ?? "https://ipfs.arkproject.dev/ipfs/",
      RUST_LOG: "INFO",
      AWS_SECRET_NAME: isProductionEnvironment
        ? "prod/ark-db-credentials"
        : "staging/ark-db-credentials",
    },
  });

  taskDefinition.addToTaskRolePolicy(
    new PolicyStatement({
      actions: ["s3:*"],
      resources: ["*"],
    })
  );

  taskDefinition.addToExecutionRolePolicy(
    new PolicyStatement({
      actions: ["s3:*"],
      resources: ["*"],
    })
  );

  new cdk.aws_ecs.FargateService(scope, `indexer-${network}`, {
    cluster: cluster,
    taskDefinition: taskDefinition,
    desiredCount: isProductionEnvironment ? 1 : 0,
  });
}
