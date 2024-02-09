import * as cdk from "aws-cdk-lib";
import { LogGroup } from "aws-cdk-lib/aws-logs";
import { Cluster } from "aws-cdk-lib/aws-ecs";

export async function deployIndexer(
  scope: cdk.Stack,
  networks: string[],
  isProductionEnvironment: boolean,
  indexerVersion: string,
  vpc: cdk.aws_ec2.IVpc
) {

  const cluster = new Cluster(scope, "arkchain-indexer", {
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
      scope,
      ecrRepository,
      network,
      indexerVersion,
      cluster
    );
  });
}

function deployIndexerServices(
  isProductionEnvironment: boolean,
  scope: cdk.Stack,
  ecrRepository: cdk.aws_ecr.IRepository,
  network: string,
  indexerVersion: string,
  cluster: cdk.aws_ecs.ICluster
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
      RUST_LOG: "DEBUG",
      ARKCHAIN_DATABASE_URL: "postgres://arkchainindexer:1J$^&R-I4VIo@arkchain-indexer-staging-dbpgarkchainc6c1584f-er1cnatb6f8y.cu3k6ojphus8.us-east-1.rds.amazonaws.com:5432/arkchainindexer",
      ARKCHAIN_RPC_PROVIDER: "https://staging.solis.arkproject.dev/"
    },
  });

  new cdk.aws_ecs.FargateService(scope, `arkchain-indexer-${network}`, {
    cluster: cluster,
    taskDefinition: taskDefinition,
    desiredCount: 1,
  });

}
