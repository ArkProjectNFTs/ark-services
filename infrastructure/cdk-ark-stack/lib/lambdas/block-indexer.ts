import * as cdk from "aws-cdk-lib";
import { RustFunction } from "cargo-lambda-cdk";
import * as iam from "aws-cdk-lib/aws-iam";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { AssetHashType } from "aws-cdk-lib";
import { IVpc, SecurityGroup, SubnetType } from "aws-cdk-lib/aws-ec2";

export function deployBlockIndexerLambda(
  scope: cdk.Stack,
  vpc: IVpc,
  lambdaSecurityGroup: SecurityGroup,
  functionName: string,
  network: string,
  tableName: string
): RustFunction {
  const blockIndexerLambda = new RustFunction(scope, functionName, {
    manifestPath: "../../ark-lambdas/lambda-block-indexer/Cargo.toml",
    environment: {
      RUST_BACKTRACE: "1",
      RUST_LOG: "info",
      RPC_PROVIDER: `https://juno.${network}.arkproject.dev`,
      INDEXER_TABLE_NAME: tableName,
      INDEXER_VERSION: "",
    },
    bundling: {
      assetHashType: AssetHashType.OUTPUT,
    },
    logRetention: RetentionDays.ONE_WEEK,
    vpc: vpc,
    vpcSubnets: {
      subnetType: SubnetType.PRIVATE_WITH_EGRESS,
    },
    securityGroups: [lambdaSecurityGroup],
    timeout: cdk.Duration.minutes(3),
  });

  let resourceArns: string[] = [];

  // Construct the necessary resource ARNs from the provided stages
  resourceArns.push(
    `arn:aws:dynamodb:${scope.region}:${scope.account}:table/${tableName}`
  );

  // Add permissions to the Lambda's role to interact with DynamoDB
  blockIndexerLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:*"],
      resources: resourceArns,
    })
  );

  // Return the RustFunction construct
  return blockIndexerLambda;
}
