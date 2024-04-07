import * as cdk from "aws-cdk-lib";
import { RustFunction } from "cargo-lambda-cdk";
import * as iam from "aws-cdk-lib/aws-iam";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { AssetHashType } from "aws-cdk-lib";
import { ISecurityGroup, IVpc, SubnetType } from "aws-cdk-lib/aws-ec2";
import { join } from "path";
import * as ssm from "aws-cdk-lib/aws-ssm";

const manifestPath = join(
  __dirname,
  "..",
  "..",
  "..",
  "..",
  "ark-lambdas",
  "lambda-block-indexer",
  "Cargo.toml"
);

export function deployBlockIndexerLambda(
  scope: cdk.Stack,
  vpc: IVpc,
  lambdaSecurityGroup: ISecurityGroup,
  functionName: string,
  network: string,
  tableName: string,
  environment: string
): RustFunction {
  const rpcProviderUri = network.includes("mainnet")
    ? `https://juno.mainnet.arkproject.dev`
    : `https://sepolia.arkproject.dev`;

  const blockIndexerLambda = new RustFunction(scope, functionName, {
    manifestPath,
    environment: {
      RUST_BACKTRACE: "1",
      RUST_LOG: "info",
      RPC_PROVIDER: rpcProviderUri,
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

  // try {
  //   new ssm.StringParameter(
  //     scope,
  //     `ark-${network}-block-indexer-function-name-${environment}`,
  //     {
  //       parameterName: `/ark/${environment}/${network}/blockIndexerFunctionName`,
  //       stringValue: blockIndexerLambda.functionName,
  //     }
  //   );
  // } catch (err) {}

  return blockIndexerLambda;
}
