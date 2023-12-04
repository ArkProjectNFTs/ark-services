import { Stack, StackProps, aws_ecr as ecr } from "aws-cdk-lib";
import { Construct } from "constructs";
import { Vpc } from "aws-cdk-lib/aws-ec2";
import { Cluster } from "aws-cdk-lib/aws-ecs";
import { deployBlockIndexerLambda } from "./lambdas/block-indexer";
import deployMetadataServices from "./metadata";
import deployIndexerServices from "./indexers";
import * as cdk from "aws-cdk-lib";
import { Table } from "aws-cdk-lib/aws-dynamodb";

export interface ArkStackProps extends cdk.StackProps {
  branch: string;
  isRelease: boolean;
  isPullRequest: boolean;
  prNumber: string;
  indexerVersion: string;
}

export class ArkProjectCdkIndexerStack extends Stack {
  constructor(scope: Construct, id: string, props: ArkStackProps) {
    super(scope, id, props);

    const indexerVersion = props.isPullRequest
      ? `PR${props.prNumber}"`
      : props.indexerVersion;

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

    const ecsTaskRole = cdk.aws_iam.Role.fromRoleArn(
      this,
      "ecsTaskExecutionRole",
      "arn:aws:iam::223605539824:role/ecsTaskExecutionRole"
    );

    ["mainnet", "testnet"].forEach((network) => {
      const tableName = props.isPullRequest
        ? "ark_project_dev"
        : `ark_project_${network}`;

      const dynamoTable = Table.fromTableName(
        this,
        `${network}DynamoTable`,
        tableName
      );

      let lambdaFunction = deployBlockIndexerLambda(
        this,
        `BlockIndexerLambda${
          network.charAt(0).toUpperCase() + network.slice(1)
        }`,
        network,
        tableName
      );

      deployMetadataServices(
        this,
        cluster,
        ecrRepository,
        network,
        dynamoTable,
        ecsTaskRole
      );

      deployIndexerServices(
        this,
        cluster,
        ecrRepository,
        network,
        indexerVersion,
        dynamoTable,
        ecsTaskRole,
        lambdaFunction.functionName,
        lambdaFunction.functionArn
      );

      new cdk.CfnOutput(this, `${network}TableOutput`, {
        value: dynamoTable.tableName,
      });
    });
  }
}
