import * as cdk from "aws-cdk-lib";
import { RustFunction } from "cargo-lambda-cdk";
import * as iam from "aws-cdk-lib/aws-iam";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { AssetHashType } from "aws-cdk-lib";

export function getContractsLambda(scope: cdk.Stack, stages: string[]) {
  const indexName = "GSI2PK-GSI2SK-index";
  // Define a RustFunction using the cargo-lambda-cdk construct
  const getContractsLambda = new RustFunction(scope, "get-contracts", {
    // Specify the path to your Rust project's Cargo.toml file
    manifestPath: "../../ark-lambdas/apigw/lambda-get-contracts/Cargo.toml",
    environment: {
      RUST_BACKTRACE: "1",
    },
    bundling: {
      assetHashType: AssetHashType.OUTPUT, // Set the assetHashType here
      // ...other bundling options if needed
    },
    logRetention: RetentionDays.ONE_DAY,
    // Additional bundling options can be specified if necessary
  });

  let resourceArns: string[] = [];

  // Construct the necessary resource ARNs from the provided stages
  for (const stage of stages) {
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}/index/${indexName}`
    );
  }

  // Add permissions to the Lambda's role to interact with DynamoDB
  getContractsLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:Query"],
      resources: resourceArns,
    })
  );

  // Return the RustFunction construct
  return getContractsLambda;
}
