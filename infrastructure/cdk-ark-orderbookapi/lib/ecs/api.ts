import * as cdk from "aws-cdk-lib";
import { LogGroup } from "aws-cdk-lib/aws-logs";
import { Cluster } from "aws-cdk-lib/aws-ecs";

export async function deployApi(
  scope: cdk.Stack,
  networks: string[],
  isProductionEnvironment: boolean,
  vpc: cdk.aws_ec2.IVpc
) {
  const cluster = new Cluster(scope, "ark-orderbook-api", {
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
      ecsSecurityGroup
    );
  });
}

function deployIndexerServices(
  isProductionEnvironment: boolean,
  scope: cdk.Stack,
  network: string,
  cluster: cdk.aws_ecs.ICluster,
  ecsSecurityGroup: cdk.aws_ec2.SecurityGroup
) {
  const logGroup = new LogGroup(scope, `/ecs/orderbook-api-${network}`, {
    removalPolicy: cdk.RemovalPolicy.DESTROY,
    retention: 7,
  });

  const taskDefinition = new cdk.aws_ecs.FargateTaskDefinition(
    scope,
    `ark-orderbook-api-${network}-task-definition`,
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

  taskDefinition.addContainer("ark_orderbook_api", {
    image: cdk.aws_ecs.ContainerImage.fromEcrRepository(
      ecrRepository,
      isProductionEnvironment
        ? "arkc-orderbook-api-production-latest"
        : "ark-orderbook-api-staging-latest"
    ),
    logging: cdk.aws_ecs.LogDrivers.awsLogs({
      streamPrefix: "ecs",
      logGroup: logGroup,
    }),
    environment: {
      RUST_LOG: "DEBUG",
      DATABASE_URL: process.env.DATABASE_URL || "defaultUrl",
    },
  });

  new cdk.aws_ecs.FargateService(scope, `ark-orderbook-api-${network}`, {
    cluster: cluster,
    taskDefinition: taskDefinition,
    desiredCount: 1,
    securityGroups: [ecsSecurityGroup],
  });
}
