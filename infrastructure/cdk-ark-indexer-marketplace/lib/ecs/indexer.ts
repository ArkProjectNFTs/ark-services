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
    // deployMetadataServices(
    //   scope,
    //   cluster,
    //   ecrRepository,
    //   network,
    //   dynamoTable,
    //   isProductionEnvironment
    // );

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

  // const blockIndexerLambdaName = ssm.StringParameter.valueForStringParameter(
  //   scope,
  //   `/ark/${
  //     isProductionEnvironment ? "production" : "staging"
  //   }/${network}/blockIndexerFunctionName`
  // );

  // const blockIndexerLambda = lambda.Function.fromFunctionName(
  //   scope,
  //   `block-indexer-${network}-${
  //     isProductionEnvironment ? "production" : "staging"
  //   }`,
  //   blockIndexerLambdaName
  // );

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
      IPFS_GATEWAY_URI: "https://ipfs.io/ipfs/",
      RPC_PROVIDER: rpcProviderUri,
      RUST_LOG: "INFO",
      AWS_SECRET_NAME: "prod/ark-db-credentials",
    },
  });

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

// function deployMetadataServices(
//   scope: cdk.Stack,
//   cluster: cdk.aws_ecs.ICluster,
//   ecrRepository: cdk.aws_ecr.IRepository,
//   network: string,
//   dynamoTable: cdk.aws_dynamodb.ITable,
//   isProductionEnvironment: boolean
// ) {
//   const capitalizedNetwork =
//     network.charAt(0).toUpperCase() + network.slice(1).toLowerCase();

//   const logGroup = new LogGroup(scope, `/ecs/ark-metadata-indexer-${network}`, {
//     removalPolicy: cdk.RemovalPolicy.DESTROY,
//     retention: 7,
//   });

//   const taskDefinition = new cdk.aws_ecs.FargateTaskDefinition(
//     scope,
//     `metadata-${network}-task-definition`,
//     {
//       memoryLimitMiB: 2048,
//       cpu: 512,
//     }
//   );

//   const rpcProviderUri = network.includes("mainnet")
//     ? `https://juno.mainnet.arkproject.dev`
//     : `https://sepolia.arkproject.dev`;

//   taskDefinition.addContainer(`ark_metadata`, {
//     image: cdk.aws_ecs.ContainerImage.fromEcrRepository(
//       ecrRepository,
//       `metadata-${isProductionEnvironment ? "production" : "staging"}-latest`
//     ),
//     logging: cdk.aws_ecs.LogDrivers.awsLogs({
//       streamPrefix: "ecs",
//       logGroup: logGroup,
//     }),
//     environment: {
//       INDEXER_TABLE_NAME: dynamoTable.tableName,
//       AWS_NFT_IMAGE_BUCKET_NAME: `ark-nft-images-${network}`,
//       RPC_PROVIDER: rpcProviderUri,
//       METADATA_IPFS_TIMEOUT_IN_SEC:
//         process.env.METADATA_IPFS_TIMEOUT_IN_SEC ?? "5",
//       METADATA_LOOP_DELAY_IN_SEC:
//         process.env.METADATA_LOOP_DELAY_IN_SEC ?? "10",
//       IPFS_GATEWAY_URI:
//         process.env.IPFS_GATEWAY_URI ?? "https://ipfs.arkproject.dev/ipfs/",
//       RUST_LOG: "INFO",
//     },
//   });

//   taskDefinition.addToTaskRolePolicy(
//     new PolicyStatement({
//       actions: ["dynamodb:*"],
//       resources: ["*"],
//     })
//   );

//   taskDefinition.addToTaskRolePolicy(
//     new PolicyStatement({
//       actions: ["s3:*"],
//       resources: ["*"],
//     })
//   );

//   dynamoTable.grantFullAccess(taskDefinition.taskRole);

//   new cdk.aws_ecs.FargateService(scope, `metadata-${network}`, {
//     cluster: cluster,
//     taskDefinition: taskDefinition,
//     desiredCount: isProductionEnvironment ? 1 : 0,
//   });
// }
