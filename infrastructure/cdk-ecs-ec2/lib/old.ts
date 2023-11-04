import * as cdk from "aws-cdk-lib";
import {
  aws_ec2 as ec2,
  aws_logs as logs,
  aws_iam as iam,
  aws_ecs as ecs,
  aws_ecr as ecr,
} from "aws-cdk-lib";
import { Construct } from "constructs";

class ArkProjectCdkEcsEc2Stack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: cdk.StackProps) {
    super(scope, id, props);

    // ECR Repository
    const ecrRepository = ecr.Repository.fromRepositoryName(
      this,
      "ArkProjectRepository",
      "ark-project-repo"
    );

    // Define the VPC
    const arkProjectVpc = new ec2.Vpc(this, "ArkProjectVpc", {
      // Define how many availability zones to use in the VPC
      maxAzs: 3, // The number of AZs to use in the VPC

      // Define the subnet configuration
      subnetConfiguration: [
        {
          cidrMask: 24, // Define the size of the subnet
          name: "Ingress",
          subnetType: ec2.SubnetType.PUBLIC, // Subnets for the NAT Gateways
        },
        {
          cidrMask: 24,
          name: "Application",
          subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS, // Use PRIVATE instead if NAT is not needed
        },
        {
          cidrMask: 28,
          name: "Database",
          subnetType: ec2.SubnetType.PRIVATE_ISOLATED, // No internet access
        },
      ],

      // Configure NAT Gateways
      natGateways: 1, // The number of NAT Gateways across all Availability Zones
    });

    const ecsSecurityGroup = new ec2.SecurityGroup(this, "EcsSecurityGroup", {
      vpc: arkProjectVpc,
      allowAllOutbound: true, // Adjust this based on your security requirements
    });

    // CloudWatch Log Group
    const logGroup = new logs.LogGroup(this, "ArkProjectMetadataLogGroup", {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
    });

    // ECS Cluster
    const arkProjectCluster = new ecs.Cluster(this, "ArkProjectCluster", {
      vpc: arkProjectVpc,
    });

    arkProjectCluster.addCapacity("ArkProjectDefaultAutoScalingGroup", {
      instanceType: new ec2.InstanceType("t3.medium"),
    });

    // ECS Task Role
    const ecsTaskRole = new iam.Role(this, "ArkProjectEcsTaskRole", {
      assumedBy: new iam.ServicePrincipal("ecs-tasks.amazonaws.com"),
    });

    // DynamoDB Table and GSI ARNs
    const tableArn =
      "arn:aws:dynamodb:us-east-1:223605539824:table/ark_project_mainnet";
    const gsiArns = [];
    for (let i = 1; i <= 5; i++) {
      gsiArns.push(`${tableArn}/index/GSI${i}PK-GSI${i}SK-index`);
    }

    // Custom policy for DynamoDB access
    const dynamoDbPolicy = new iam.Policy(this, "ArkProjectDynamoDbPolicy", {
      statements: [
        new iam.PolicyStatement({
          effect: iam.Effect.ALLOW,
          actions: [
            "dynamodb:GetItem",
            "dynamodb:Query",
            "dynamodb:Scan",
            "dynamodb:PutItem",
            "dynamodb:UpdateItem",
            "dynamodb:DeleteItem",
            "dynamodb:BatchWriteItem",
            "dynamodb:BatchGetItem",
            "dynamodb:ConditionCheckItem",
          ],
          resources: [tableArn, ...gsiArns],
        }),
      ],
    });

    // Attach the custom policy to the task role
    ecsTaskRole.attachInlinePolicy(dynamoDbPolicy);

    // Attach the AWS managed DynamoDB Full Access policy
    ecsTaskRole.addManagedPolicy(
      iam.ManagedPolicy.fromAwsManagedPolicyName("AmazonDynamoDBFullAccess")
    );

    // ECS Task Execution Role
    const ecsTaskExecutionRole = new iam.Role(
      this,
      "ArkProjectEcsTaskExecutionRole",
      {
        assumedBy: new iam.ServicePrincipal("ecs-tasks.amazonaws.com"),
      }
    );

    ecsTaskExecutionRole.addManagedPolicy(
      iam.ManagedPolicy.fromAwsManagedPolicyName(
        "service-role/AmazonECSTaskExecutionRolePolicy"
      )
    );

    // ECS Task Definition with Task Role and Execution Role
    const arkProjectMetadataTaskDefinition = new ecs.Ec2TaskDefinition(
      this,
      "ArkProjectMetadataTaskDef",
      {
        taskRole: ecsTaskRole,
        executionRole: ecsTaskExecutionRole,
      }
    );

    // ECS Container Definition
    arkProjectMetadataTaskDefinition.addContainer(
      "ArkProjectMetadataContainer",
      {
        image: ecs.ContainerImage.fromEcrRepository(ecrRepository),
        memoryLimitMiB: 256,
        environment: {
          INDEXER_TABLE_NAME: process.env.INDEXER_TABLE_NAME ?? "default",
          AWS_NFT_IMAGE_BUCKET_NAME:
            process.env.AWS_NFT_IMAGE_BUCKET_NAME ?? "default",
          RPC_PROVIDER: process.env.RPC_PROVIDER ?? "default",
          METADATA_IPFS_TIMEOUT_IN_SEC:
            process.env.METADATA_IPFS_TIMEOUT_IN_SEC ?? "default",
          METADATA_LOOP_DELAY_IN_SEC:
            process.env.METADATA_LOOP_DELAY_IN_SEC ?? "default",
          IPFS_GATEWAY_URI: process.env.IPFS_GATEWAY_URI ?? "default",
          RUST_LOG: "INFO",
        },
        logging: ecs.LogDrivers.awsLogs({
          logGroup: logGroup,
          streamPrefix: "ArkProjectMetadata",
        }),
      }
    );

    // ECS Service
    new ecs.Ec2Service(this, "ArkProjectMetadataService", {
      cluster: arkProjectCluster,
      taskDefinition: arkProjectMetadataTaskDefinition,
      securityGroups: [ecsSecurityGroup],
      vpcSubnets: {
        subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS, // or PUBLIC if you want your tasks to be in a public subnet
      },
    });
  }
}

export { ArkProjectCdkEcsEc2Stack };
