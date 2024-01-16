import * as cdk from "aws-cdk-lib";
import { RustFunction } from "cargo-lambda-cdk";
import * as iam from "aws-cdk-lib/aws-iam";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { AssetHashType } from "aws-cdk-lib";
import { IVpc, SecurityGroup, SubnetType } from "aws-cdk-lib/aws-ec2";
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
  "lambda-get-contract-tokens",
  "Cargo.toml"
);

console.log("=> manifestPath", manifestPath);

export function getContractTokensLambda(
  scope: cdk.Stack,
  vpc: IVpc,
  lambdaSecurityGroup: SecurityGroup,
  stages: string[],
  tableNamePrefix: string
) {
  const indexName = "GSI1PK-GSI1SK-index";

  const getContractTokensLambda = new RustFunction(
    scope,
    "get-contract-tokens",
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
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/${tableNamePrefix}_${stage}/index/${indexName}`
    );
  }

  // Add permissions to the Lambda's role to interact with DynamoDB
  getContractTokensLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:*"],
      resources: resourceArns,
    })
  );

  // Return the RustFunction construct
  return getContractTokensLambda;
}
