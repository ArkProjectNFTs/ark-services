import * as cdk from "aws-cdk-lib";
import * as ec2 from "aws-cdk-lib/aws-ec2";
import * as ecs from "aws-cdk-lib/aws-ecs";
import * as iam from "aws-cdk-lib/aws-iam";
import { StackProps } from "aws-cdk-lib";
import { Construct } from "constructs";

export class CdkMetadataStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);

    // Create a VPC
    const vpc = new ec2.Vpc(this, "MyVpc", { maxAzs: 2 });

    // Create an ECS Cluster
    const cluster = new ecs.Cluster(this, "ark-indexers", { vpc });

    // Create a task definition with CloudWatch Logs
    const logging = new ecs.AwsLogDriver({ streamPrefix: "myapp" });

    // const executionRole = new iam.Role(this, "EcsTaskExecutionRole", {
    //   assumedBy: new iam.ServicePrincipal("ecs-tasks.amazonaws.com"),
    // });

    const taskDef = new ecs.FargateTaskDefinition(
      this,
      "ark-metadata-indexer-mainnet",
      {
        cpu: 1,
      }
    );

    taskDef.addContainer("AppContainer", {
      image: ecs.ContainerImage.fromRegistry("amazon/amazon-ecs-sample"),
      logging,
    });

    // Instantiate an ECS Service with cluster and task definition
    new ecs.FargateService(this, "FargateService", {
      cluster,
      taskDefinition: taskDef,
    });
  }
}
