import { Stack, StackProps, CfnOutput, RemovalPolicy, aws_ecs as ecs, aws_ecr as ecr, aws_iam as iam, aws_ec2 as ec2 } from "aws-cdk-lib";
import { Construct } from "constructs";
import { Table } from "aws-cdk-lib/aws-dynamodb";
import { Vpc } from "aws-cdk-lib/aws-ec2";
import { Cluster } from "aws-cdk-lib/aws-ecs";
import { PolicyStatement } from "aws-cdk-lib/aws-iam";
import { LogGroup } from "aws-cdk-lib/aws-logs";

export class ArkProjectCdkEcsFargateStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);

    // Networks to deploy tasks for
    const networks = ["mainnet", "testnet"];

    // Common resources that can be shared across both mainnet and testnet
    const vpc = new Vpc(this, "ArkVpc", {
      maxAzs: 3,
    });

    const cluster = new Cluster(this, "ArkCluster", {
      vpc: vpc,
    });

    const ecrRepository = ecr.Repository.fromRepositoryName(
      this,
      "ArkProjectRepository",
      "ark-project-repo"
    );

    networks.forEach(network => {
      const tableName = `ark_project_${network}`;
      const dynamoTable = Table.fromTableName(this, `${network}DynamoTable`, tableName);

      const logGroup = new LogGroup(this, `${network}ArkLogGroup`, {
        removalPolicy: RemovalPolicy.DESTROY,
      });

      const taskDefinition = new ecs.FargateTaskDefinition(this, `${network}ArkMetadataTaskDef`, {
        memoryLimitMiB: 512,
        cpu: 256,
      });

      taskDefinition.addContainer(`${network}ArkMetadataIndexerContainer`, {
        image: ecs.ContainerImage.fromEcrRepository(ecrRepository),
        logging: ecs.LogDrivers.awsLogs({
          streamPrefix: `${network}ArkMetadataIndexer`,
          logGroup: logGroup,
        }),
        environment: {
          INDEXER_TABLE_NAME: tableName,
          AWS_NFT_IMAGE_BUCKET_NAME: `ark-nft-images-${network}`,
          RPC_PROVIDER: `https://juno.${network}.arkproject.dev`,
          METADATA_IPFS_TIMEOUT_IN_SEC: process.env.METADATA_IPFS_TIMEOUT_IN_SEC ?? "default-timeout",
          METADATA_LOOP_DELAY_IN_SEC: process.env.METADATA_LOOP_DELAY_IN_SEC ?? "default-delay",
          IPFS_GATEWAY_URI: process.env.IPFS_GATEWAY_URI ?? "default-gateway",
          RUST_LOG: "INFO",
        },
      });

      taskDefinition.addToTaskRolePolicy(new PolicyStatement({
        actions: ["dynamodb:*"],
        resources: ["*"],
      }));

      taskDefinition.addToTaskRolePolicy(new PolicyStatement({
        actions: ["s3:*"],
        resources: ["*"],
      }));

      dynamoTable.grantFullAccess(taskDefinition.taskRole);

      new ecs.FargateService(this, `${network}ArkMetadataFargateService`, {
        cluster: cluster,
        taskDefinition: taskDefinition,
        desiredCount: 1,
      });

      new CfnOutput(this, `${network}TableNameOutput`, {
        value: dynamoTable.tableName
      });
    });
  }
}
