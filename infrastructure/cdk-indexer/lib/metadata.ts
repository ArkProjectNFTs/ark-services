import * as cdk from "aws-cdk-lib";
import { RemovalPolicy, aws_ecs as ecs } from "aws-cdk-lib";
import { PolicyStatement } from "aws-cdk-lib/aws-iam";
import { LogGroup } from "aws-cdk-lib/aws-logs";

export default function deployMetadataServices(
  scope: cdk.Stack,
  cluster: cdk.aws_ecs.ICluster,
  ecrRepository: cdk.aws_ecr.IRepository,
  network: string,
  dynamoTable: cdk.aws_dynamodb.ITable,
  ecsTaskRole: cdk.aws_iam.IRole
) {
  const capitalizedNetwork =
    network.charAt(0).toUpperCase() + network.slice(1).toLowerCase();

  const logGroup = new LogGroup(scope, `/ecs/ark-metadata-indexer-${network}`, {
    removalPolicy: RemovalPolicy.DESTROY,
    retention: 7,
  });

  const taskDefinition = new ecs.FargateTaskDefinition(
    scope,
    `Metadata${capitalizedNetwork}TaskDefinition`,
    {
      memoryLimitMiB: 2048,
      cpu: 512,
      taskRole: ecsTaskRole,
    }
  );

  taskDefinition.addContainer(`ark_metadata`, {
    image: ecs.ContainerImage.fromEcrRepository(
      ecrRepository,
      "metadata-latest"
    ),
    logging: ecs.LogDrivers.awsLogs({
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

  new ecs.FargateService(scope, `Metadata${capitalizedNetwork}`, {
    cluster: cluster,
    taskDefinition: taskDefinition,
    desiredCount: 1,
  });
}
