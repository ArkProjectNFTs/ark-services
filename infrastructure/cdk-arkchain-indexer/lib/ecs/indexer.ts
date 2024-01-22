import * as cdk from "aws-cdk-lib";
import { IVpc } from "aws-cdk-lib/aws-ec2";
import { Cluster } from "aws-cdk-lib/aws-ecs";
import { Table } from "aws-cdk-lib/aws-dynamodb";
import { LogGroup } from "aws-cdk-lib/aws-logs";
import { PolicyStatement } from "aws-cdk-lib/aws-iam";
import * as lambda from "aws-cdk-lib/aws-lambda";
import * as ssm from "aws-cdk-lib/aws-ssm";

export async function deployIndexer(
  scope: cdk.Stack,
  networks: string[],
  vpc: IVpc,
  isProductionEnvironment: boolean,
  indexerVersion: string
) {
  const cluster = new Cluster(scope, "indexers", {
    vpc: vpc,
  });

  const ecrRepository = cdk.aws_ecr.Repository.fromRepositoryName(
    scope,
    "ArkProjectRepository",
    "ark-project-repo"
  );

  networks.forEach((network) => {
    const tableName = isProductionEnvironment
      ? `ark_project_${network}`
      : `ark_project_staging_${network}`;


    deployIndexerServices(
      isProductionEnvironment,
      scope,
      ecrRepository,
      network,
      indexerVersion,
    );
  });
}

function deployIndexerServices(
  isProductionEnvironment: boolean,
  scope: cdk.Stack,
  ecrRepository: cdk.aws_ecr.IRepository,
  network: string,
  indexerVersion: string,
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
      HEAD_OF_CHAIN: "true",
      INDEXER_VERSION: indexerVersion,
      RUST_LOG: "INFO",
    },
  });


}
