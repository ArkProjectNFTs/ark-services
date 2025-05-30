import * as cdk from "aws-cdk-lib";
import { RustFunction } from "cargo-lambda-cdk";
import * as iam from "aws-cdk-lib/aws-iam";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { AssetHashType } from "aws-cdk-lib";
import { ISecurityGroup, IVpc, SubnetType } from "aws-cdk-lib/aws-ec2";
import { join } from "path";

const manifestPath = join(
  __dirname,
  "..",
  "..",
  "..",
  "..",
  "..",
  "ark-lambdas",
  "apigw",
  "lambda-get-owner-contracts",
  "Cargo.toml"
);

export function getOwnerContractsLambda(
  scope: cdk.Stack,
  vpc: IVpc,
  lambdaSecurityGroup: ISecurityGroup,
  stages: string[],
  tableNamePrefix: string
) {
  const indexName = "GSI2PK-GSI2SK-index";

  const getOwnerContractsLambda = new RustFunction(
    scope,
    "get-owner-contracts",
    {
      manifestPath,
      environment: {
        RUST_BACKTRACE: "1",
      },
      bundling: {
        assetHashType: AssetHashType.OUTPUT,
      },
      logRetention: RetentionDays.ONE_DAY,
      vpc: vpc,
      vpcSubnets: {
        subnetType: SubnetType.PRIVATE_WITH_EGRESS,
      },
      securityGroups: [lambdaSecurityGroup],
      timeout: cdk.Duration.seconds(10),
    }
  );

  let resourceArns: string[] = [];

  // Construct the necessary resource ARNs from the provided stages
  for (const stage of stages) {
    const baseTableArn = `arn:aws:dynamodb:${scope.region}:${scope.account}:table/${tableNamePrefix}_${stage}`;
    // ARN for index - used with dynamodb:Query
    resourceArns.push(`${baseTableArn}/index/${indexName}`);
    // ARN for table - used with dynamodb:GetItem and dynamodb:PutItem
    resourceArns.push(baseTableArn);
  }

  // Add permissions to the Lambda's role to interact with DynamoDB
  getOwnerContractsLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:*"],
      resources: resourceArns,
    })
  );

  // Return the RustFunction construct
  return getOwnerContractsLambda;
}
