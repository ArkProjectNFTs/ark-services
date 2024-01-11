import * as cdk from "aws-cdk-lib";
import { IVpc, SecurityGroup } from "aws-cdk-lib/aws-ec2";
import { Cluster } from "aws-cdk-lib/aws-ecs";
import { Table } from "aws-cdk-lib/aws-dynamodb";
import { deployBlockIndexerLambda } from "../lambdas/block-indexer";
import { LogGroup } from "aws-cdk-lib/aws-logs";
import { PolicyStatement } from "aws-cdk-lib/aws-iam";

export async function deployIndexer(
  scope: cdk.Stack,
  vpc: IVpc,
  lambdaSecurityGroup: SecurityGroup,
  isProductionEnvironment: boolean,
  indexerVersion: string
) {
  const cluster = new Cluster(scope, "Indexers", {
    vpc: vpc,
  });

  const ecrRepository = cdk.aws_ecr.Repository.fromRepositoryName(
    scope,
    "ArkProjectRepository",
    "ark-project-repo"
  );

  ["mainnet", "testnet"].forEach((network) => {
    const tableName = isProductionEnvironment
      ? `ark_project_${network}`
      : `ark_project_staging_${network}`;

    const dynamoTable = Table.fromTableName(
      scope,
      `${network}DynamoTable`,
      tableName
    );

    let lambdaFunction = deployBlockIndexerLambda(
      scope,
      vpc,
      lambdaSecurityGroup,
      `BlockIndexerLambda${network.charAt(0).toUpperCase() + network.slice(1)}`,
      network,
      tableName
    );

    deployMetadataServices(scope, cluster, ecrRepository, network, dynamoTable);

    deployIndexerServices(
      isProductionEnvironment,
      scope,
      cluster,
      ecrRepository,
      network,
      indexerVersion,
      dynamoTable,
      lambdaFunction.functionName,
      lambdaFunction.functionArn
    );

    new cdk.CfnOutput(scope, `${network}TableOutput`, {
      value: dynamoTable.tableName,
    });
  });
}

function deployIndexerServices(
  isProductionEnvironment: boolean,
  scope: cdk.Stack,
  cluster: cdk.aws_ecs.ICluster,
  ecrRepository: cdk.aws_ecr.IRepository,
  network: string,
  indexerVersion: string,
  dynamoTable: cdk.aws_dynamodb.ITable,
  functionName: string,
  functionArn: string
) {
  const capitalizedNetwork =
    network.charAt(0).toUpperCase() + network.slice(1).toLowerCase();

  const logGroup = new LogGroup(scope, `/ecs/ark-indexer-${network}`, {
    removalPolicy: cdk.RemovalPolicy.DESTROY,
    retention: 7,
  });

  const taskDefinition = new cdk.aws_ecs.FargateTaskDefinition(
    scope,
    `Indexer${capitalizedNetwork}TaskDefinition`,
    {
      memoryLimitMiB: 2048,
      cpu: 512,
    }
  );

  taskDefinition.addContainer("ark_indexer", {
    image: cdk.aws_ecs.ContainerImage.fromEcrRepository(
      ecrRepository,
      isProductionEnvironment
        ? "indexer-production-latest"
        : "indexer-staging-latest"
    ),
    logging: cdk.aws_ecs.LogDrivers.awsLogs({
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
      actions: ["logs:CreateLogStream", "logs:PutLogEvents"],
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

  new cdk.aws_ecs.FargateService(scope, `HeadOfChain${capitalizedNetwork}`, {
    cluster: cluster,
    taskDefinition: taskDefinition,
    desiredCount: 1,
  });
}

function deployMetadataServices(
  scope: cdk.Stack,
  cluster: cdk.aws_ecs.ICluster,
  ecrRepository: cdk.aws_ecr.IRepository,
  network: string,
  dynamoTable: cdk.aws_dynamodb.ITable
) {
  const capitalizedNetwork =
    network.charAt(0).toUpperCase() + network.slice(1).toLowerCase();

  const logGroup = new LogGroup(scope, `/ecs/ark-metadata-indexer-${network}`, {
    removalPolicy: cdk.RemovalPolicy.DESTROY,
    retention: 7,
  });

  const taskDefinition = new cdk.aws_ecs.FargateTaskDefinition(
    scope,
    `Metadata${capitalizedNetwork}TaskDefinition`,
    {
      memoryLimitMiB: 2048,
      cpu: 512,
    }
  );

  taskDefinition.addContainer(`ark_metadata`, {
    image: cdk.aws_ecs.ContainerImage.fromEcrRepository(
      ecrRepository,
      "metadata-latest"
    ),
    logging: cdk.aws_ecs.LogDrivers.awsLogs({
      streamPrefix: "ecs",
      logGroup: logGroup,
    }),
    environment: {
      INDEXER_TABLE_NAME: dynamoTable.tableName,
      AWS_NFT_IMAGE_BUCKET_NAME: `ark-nft-images-${network}`,
      RPC_PROVIDER: `https://juno.${network}.arkproject.dev`,
      METADATA_IPFS_TIMEOUT_IN_SEC:
        process.env.METADATA_IPFS_TIMEOUT_IN_SEC ?? "5",
      METADATA_LOOP_DELAY_IN_SEC:
        process.env.METADATA_LOOP_DELAY_IN_SEC ?? "10",
      IPFS_GATEWAY_URI:
        process.env.IPFS_GATEWAY_URI ?? "https://ipfs.arkproject.dev/ipfs/",
      RUST_LOG: "INFO",
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

  dynamoTable.grantFullAccess(taskDefinition.taskRole);

  new cdk.aws_ecs.FargateService(scope, `Metadata${capitalizedNetwork}`, {
    cluster: cluster,
    taskDefinition: taskDefinition,
    desiredCount: 1,
  });
}
