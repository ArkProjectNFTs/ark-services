import * as cdk from "aws-cdk-lib";
import { Table } from "aws-cdk-lib/aws-dynamodb";
import { CfnOutput, RemovalPolicy, aws_ecs as ecs } from "aws-cdk-lib";
import { PolicyStatement } from "aws-cdk-lib/aws-iam";
import { LogGroup } from "aws-cdk-lib/aws-logs";

export default function deployIndexerServices(
  scope: cdk.Stack,
  cluster: cdk.aws_ecs.ICluster,
  ecrRepository: cdk.aws_ecr.IRepository,
  network: string,
  indexerVersion: string,
  dynamoTable: cdk.aws_dynamodb.ITable,
  ecsTaskRole: cdk.aws_iam.IRole,
  functionName: string,
  functionArn: string
) {
  const capitalizedNetwork =
    network.charAt(0).toUpperCase() + network.slice(1).toLowerCase();

  const logGroup = new LogGroup(scope, `/ecs/ark-indexer-${network}`, {
    removalPolicy: RemovalPolicy.DESTROY,
    retention: 7,
  });

  const taskDefinition = new ecs.FargateTaskDefinition(
    scope,
    `Indexer${capitalizedNetwork}TaskDefinition`,
    {
      memoryLimitMiB: 2048,
      cpu: 512,
      taskRole: ecsTaskRole,
    }
  );

  taskDefinition.addContainer("ark_indexer", {
    image: ecs.ContainerImage.fromEcrRepository(
      ecrRepository,
      "indexer-latest"
    ),
    logging: ecs.LogDrivers.awsLogs({
      streamPrefix: "ecs",
      logGroup: logGroup,
    }),
    environment: {
      HEAD_OF_CHAIN: "true",
      INDEXER_TABLE_NAME: dynamoTable.tableName,
      INDEXER_VERSION: indexerVersion,
      IPFS_GATEWAY_URI: "https://ipfs.arkproject.dev",
      RPC_PROVIDER: `https://juno.${network}.arkproject.dev`,
      RUST_LOG: "INFO",
      BLOCK_INDEXER_FUNCTION_NAME: functionName,
    },
  });

  taskDefinition.addToTaskRolePolicy(
    new PolicyStatement({
      actions: ["dynamodb:*"],
      resources: ["*"],
    })
  );

  taskDefinition.addToTaskRolePolicy(
    new PolicyStatement({
      actions: ["s3:*"],
      resources: ["*"],
    })
  );

  taskDefinition.addToTaskRolePolicy(
    new PolicyStatement({
      actions: [
        "lambda:InvokeAsync",
        "lambda:InvokeFunction",
        "lambda:InvokeFunctionUrl",
      ],
      resources: [functionArn],
    })
  );

  dynamoTable.grantFullAccess(taskDefinition.taskRole);

  new ecs.FargateService(scope, `HeadOfChain${capitalizedNetwork}`, {
    cluster: cluster,
    taskDefinition: taskDefinition,
    desiredCount: 1,
  });
}
