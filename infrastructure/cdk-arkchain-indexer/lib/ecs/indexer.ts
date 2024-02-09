import * as cdk from "aws-cdk-lib";
import { LogGroup } from "aws-cdk-lib/aws-logs";
import { Cluster } from "aws-cdk-lib/aws-ecs";
import { DatabaseInstance } from "aws-cdk-lib/aws-rds";

export async function deployIndexer(
  scope: cdk.Stack,
  networks: string[],
  isProductionEnvironment: boolean,
  vpc: cdk.aws_ec2.IVpc,
  dbEndpointAddress: string
) {
  const cluster = new Cluster(scope, "arkchain-indexer", {
    vpc: vpc,
  });

  const ecsSecurityGroup = new cdk.aws_ec2.SecurityGroup(
    scope,
    "ECSSecurityGroup",
    {
      vpc,
      description: "Security group for ECS tasks",
    }
  );

  networks.forEach((network) => {
    deployIndexerServices(
      isProductionEnvironment,
      scope,
      network,
      cluster,
      dbEndpointAddress,
      ecsSecurityGroup
    );
  });
}

function deployIndexerServices(
  isProductionEnvironment: boolean,
  scope: cdk.Stack,
  network: string,
  cluster: cdk.aws_ecs.ICluster,
  dbEndpointAddress: string,
  ecsSecurityGroup: cdk.aws_ec2.SecurityGroup
) {
  const logGroup = new LogGroup(scope, `/ecs/arkchain-indexer-${network}`, {
    removalPolicy: cdk.RemovalPolicy.DESTROY,
    retention: 7,
  });

  const taskDefinition = new cdk.aws_ecs.FargateTaskDefinition(
    scope,
    `arkchain-indexer-${network}-task-definition`,
    {
      memoryLimitMiB: 2048,
      cpu: 512,
    }
  );

  const ecrRepository = cdk.aws_ecr.Repository.fromRepositoryName(
    scope,
    "ArkProjectRepository",
    "ark-project-repo"
  );

  taskDefinition.addContainer("arkchain_indexer", {
    image: cdk.aws_ecs.ContainerImage.fromEcrRepository(
      ecrRepository,
      isProductionEnvironment
        ? "arkchain-indexer-production-latest"
        : "arkchain-indexer-staging-latest"
    ),
    logging: cdk.aws_ecs.LogDrivers.awsLogs({
      streamPrefix: "ecs",
      logGroup: logGroup,
    }),
    environment: {
      //PGSSLMODE: "require",
      RUST_LOG: "DEBUG",
      ARKCHAIN_DATABASE_URL: `postgres://${
        process.env.DB_USERNAME || "defaultUsername"
      }:${
        process.env.DB_PASSWORD || "defaultPassword"
      }@${dbEndpointAddress}:5432/arkchainindexer`,
      ARKCHAIN_RPC_PROVIDER:  process.env.ARKCHAIN_RPC_PROVIDER || "defaultProvider",
    },
  });

  new cdk.aws_ecs.FargateService(scope, `arkchain-indexer-${network}`, {
    cluster: cluster,
    taskDefinition: taskDefinition,
    desiredCount: 1,
    securityGroups: [ecsSecurityGroup],
  });
}
