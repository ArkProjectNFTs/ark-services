import {
  Stack,
  StackProps,
  CfnOutput,
  RemovalPolicy,
  aws_ecs as ecs,
  aws_ecr as ecr,
  aws_iam as iam,
  aws_ec2 as ec2,
} from "aws-cdk-lib";
import { Construct } from "constructs";
import { Table } from "aws-cdk-lib/aws-dynamodb";
import {
  GatewayVpcEndpointAwsService,
  CfnVPCEndpoint,
  Vpc,
} from "aws-cdk-lib/aws-ec2";
import { Cluster } from "aws-cdk-lib/aws-ecs";
import { Effect, PolicyStatement } from "aws-cdk-lib/aws-iam";
import { LogGroup } from "aws-cdk-lib/aws-logs";
import path = require("path");

export class ArkProjectCdkEcsFargateStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);

    // Existing DynamoDB table reference
    const dynamoTable = Table.fromTableName(
      this,
      "DynamoTable",
      process.env.INDEXER_TABLE_NAME ?? "default-table-name"
    );

    const vpc = new Vpc(this, "ArkVpc", {
      maxAzs: 3,
    });

    const dynamoDbEndpoint = vpc.addGatewayEndpoint("DynamoDbEndpoint", {
      service: ec2.GatewayVpcEndpointAwsService.DYNAMODB,
    });

    dynamoDbEndpoint.addToPolicy(
      new PolicyStatement({
        effect: Effect.ALLOW,
        principals: [new iam.AnyPrincipal()],
        actions: ["dynamodb:*"],
        resources: ["*"],
      })
    );

    const cluster = new Cluster(this, "ArkCluster", {
      vpc: vpc,
    });

    // Log Group for container logs
    const logGroup = new LogGroup(this, "ArkLogGroup", {
      removalPolicy: RemovalPolicy.DESTROY,
    });

    // Create an ECR repository reference (assumes it already exists)
    const ecrRepository = ecr.Repository.fromRepositoryName(
      this,
      "ArkProjectRepository",
      "ark-project-repo"
    );

    // Define the Fargate task definition
    const taskDefinition = new ecs.FargateTaskDefinition(
      this,
      "ArkMetadataTaskDef",
      {
        memoryLimitMiB: 512,
        cpu: 256,
      }
    );

    // Add container to the task definition
    taskDefinition.addContainer("ArkMetadataIndexerContainer", {
      image: ecs.ContainerImage.fromEcrRepository(ecrRepository),
      logging: ecs.LogDrivers.awsLogs({
        streamPrefix: "ArkMetadataIndexer",
        logGroup: logGroup,
      }),
      environment: {
        INDEXER_TABLE_NAME: process.env.INDEXER_TABLE_NAME ?? "default",
        AWS_NFT_IMAGE_BUCKET_NAME:
          process.env.AWS_NFT_IMAGE_BUCKET_NAME ?? "default-bucket-name",
        RPC_PROVIDER: process.env.RPC_PROVIDER ?? "default-provider",
        METADATA_IPFS_TIMEOUT_IN_SEC:
          process.env.METADATA_IPFS_TIMEOUT_IN_SEC ?? "default-timeout",
        METADATA_LOOP_DELAY_IN_SEC:
          process.env.METADATA_LOOP_DELAY_IN_SEC ?? "default-delay",
        IPFS_GATEWAY_URI: process.env.IPFS_GATEWAY_URI ?? "default-gateway",
        RUST_LOG: "INFO",
      },
    });

    // Ensure the IAM Role for the ECS Task has the necessary permissions
    taskDefinition.addToTaskRolePolicy(
      new PolicyStatement({
        actions: ["dynamodb:*"],
        resources: ["*"],
      })
    );

    // DynamoDB table permissions for the task role
    dynamoTable.grantFullAccess(taskDefinition.taskRole);

    // Create the Fargate service
    new ecs.FargateService(this, "ArkMetadataFargateService", {
      cluster: cluster,
      taskDefinition: taskDefinition,
      desiredCount: 1,
    });

    // Outputs
    new CfnOutput(
      this,
      process.env.INDEXER_TABLE_NAME ?? "default-table-name",
      { value: dynamoTable.tableName }
    );
  }
}
