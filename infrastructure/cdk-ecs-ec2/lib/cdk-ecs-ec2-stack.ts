// Import necessary CDK libraries
import * as cdk from "aws-cdk-lib";
import {
  aws_ec2 as ec2,
  aws_logs as logs,
  aws_iam as iam,
  aws_ecs as ecs,
  aws_ecr as ecr,
} from "aws-cdk-lib";
import { Construct } from "constructs";

// Define the stack class
class ArkProjectCdkEcsEc2Stack extends cdk.Stack {
  // The constructor for the stack
  constructor(scope: Construct, id: string, props: cdk.StackProps) {
    // Call the parent constructor
    super(scope, id, props);

    // Create an ECR repository reference (assumes it already exists)
    const ecrRepository = ecr.Repository.fromRepositoryName(
      this,
      "ArkProjectRepository",
      "ark-project-repo"
    );

    // Create a VPC with a specific configuration
    const arkProjectVpc = new ec2.Vpc(this, "ArkProjectVpc", {
      ipAddresses: ec2.IpAddresses.cidr("192.168.1.0/24"),
      maxAzs: 3,
      subnetConfiguration: [
        {
          cidrMask: 28,
          name: "public-subnet",
          subnetType: ec2.SubnetType.PUBLIC,
        },
        {
          cidrMask: 28, // Adjusted the cidrMask to reflect a /28 subnet
          name: "private-subnet",
          subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS,
        },
        {
          cidrMask: 28, // Adjusted the cidrMask to reflect a /28 subnet
          name: "isolated-subnet",
          subnetType: ec2.SubnetType.PRIVATE_ISOLATED,
        },
      ],
      enableDnsHostnames: true,
      enableDnsSupport: true,
      natGateways: 1,
    });

    // Create a security group for the ECS tasks
    const ecsSecurityGroup = new ec2.SecurityGroup(this, "EcsSecurityGroup", {
      vpc: arkProjectVpc,
      allowAllOutbound: true, // Tasks can initiate outbound traffic to any destination
    });

    ecsSecurityGroup.addIngressRule(
      ec2.Peer.anyIpv4(),
      ec2.Port.allTraffic(),
      "Allow all traffic"
    );

    // Create a CloudWatch Log Group for ECS task logs
    const logGroup = new logs.LogGroup(this, "ArkProjectMetadataLogGroup", {
      removalPolicy: cdk.RemovalPolicy.DESTROY, // Clean up the log group when the stack is deleted
    });

    // Create an ECS Cluster within the created VPC
    const arkProjectCluster = new ecs.Cluster(this, "ArkProjectCluster", {
      vpc: arkProjectVpc, // Reference to the VPC created above
    });

    // Add default Auto Scaling group capacity to the ECS Cluster
    arkProjectCluster.addCapacity("ArkProjectDefaultAutoScalingGroup", {
      instanceType: new ec2.InstanceType("t3.medium"), // Specify the EC2 instance type for the ECS cluster
    });

    // Define an IAM role for ECS tasks
    const ecsTaskRole = new iam.Role(this, "ArkProjectEcsTaskRole", {
      assumedBy: new iam.ServicePrincipal("ecs-tasks.amazonaws.com"), // Only ECS tasks can assume this role
    });

    // Adding DynamoDB access to the ECS Task Role
    const tableArn =
      "arn:aws:dynamodb:us-east-1:123456789012:table/ark_project_mainnet"; // Replace with actual ARN
    const gsiArns = [];
    for (let i = 1; i <= 5; i++) {
      gsiArns.push(`${tableArn}/index/GSI${i}PK-GSI${i}SK-index`);
    }

    // Attach the AWS managed policy for DynamoDB Full Access (this is generally not recommended due to broad permissions)
    ecsTaskRole.addManagedPolicy(
      iam.ManagedPolicy.fromAwsManagedPolicyName("AmazonDynamoDBFullAccess")
    );

    // Define an IAM role for ECS Task Execution
    const ecsTaskExecutionRole = new iam.Role(
      this,
      "ArkProjectEcsTaskExecutionRole",
      {
        assumedBy: new iam.ServicePrincipal("ecs-tasks.amazonaws.com"), // Only ECS tasks can assume this role
      }
    );

    // Attach the managed policy for ECS Task Execution which provides permissions needed by the ECS agent
    ecsTaskExecutionRole.addManagedPolicy(
      iam.ManagedPolicy.fromAwsManagedPolicyName(
        "service-role/AmazonECSTaskExecutionRolePolicy"
      )
    );

    // Define the ECS Task Definition with the Task Role and Execution Role
    const arkProjectMetadataTaskDefinition = new ecs.Ec2TaskDefinition(
      this,
      "ArkProjectMetadataTaskDef",
      {
        networkMode: ecs.NetworkMode.AWS_VPC,
        taskRole: ecsTaskRole, // IAM role for authorizing AWS service calls from your task
        executionRole: ecsTaskExecutionRole, // IAM role that ECS uses to pull images and store logs
      }
    );

    // Add a container definition to the task definition
    arkProjectMetadataTaskDefinition.addContainer(
      "ArkProjectMetadataContainer",
      {
        image: ecs.ContainerImage.fromEcrRepository(ecrRepository), // Use the image from ECR
        memoryLimitMiB: 256, // The amount of memory to allocate to the container
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

    const dynamoDbEndpoint = arkProjectVpc.addGatewayEndpoint(
      "DynamoDbEndpoint",
      {
        service: ec2.GatewayVpcEndpointAwsService.DYNAMODB,
      }
    );

    dynamoDbEndpoint.addToPolicy(
      new iam.PolicyStatement({
        principals: [new iam.AnyPrincipal()],
        effect: iam.Effect.ALLOW,
        actions: ["dynamodb:*"],
        resources: ["*"],
      })
    );

    // Create an ECS Service that runs the specified task definition
    new ecs.Ec2Service(this, "ArkProjectMetadataService", {
      cluster: arkProjectCluster, // Reference to the ECS cluster
      taskDefinition: arkProjectMetadataTaskDefinition, // Reference to the task definition
      securityGroups: [ecsSecurityGroup], // Reference to the security group
      vpcSubnets: {
        subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS,
      },
    });
  }
}

// Export the stack class
export { ArkProjectCdkEcsEc2Stack };
