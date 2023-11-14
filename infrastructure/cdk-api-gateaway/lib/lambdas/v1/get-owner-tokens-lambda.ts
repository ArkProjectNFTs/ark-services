import * as cdk from "aws-cdk-lib";
import { RustFunction } from "cargo-lambda-cdk";
import * as iam from "aws-cdk-lib/aws-iam";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { AssetHashType } from "aws-cdk-lib";

export function getOwnerTokensLambda(scope: cdk.Stack, stages: string[]) {
  const indexName = "GSI2PK-GSI2SK-index";
  const getOwnerTokensLambda = new RustFunction(scope, "get-owner-tokens", {
    manifestPath: "../../ark-lambdas/apigw/lambda-get-owner-tokens/Cargo.toml",
    environment: {
      RUST_BACKTRACE: "1",
    },
    bundling: {
      assetHashType: AssetHashType.OUTPUT, // Set the assetHashType here
      // ...other bundling options if needed
    },
    logRetention: RetentionDays.ONE_DAY,
  });

  let resourceArns: string[] = [];

  for (const stage of stages) {
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}/index/${indexName}`
    );
  }

  getOwnerTokensLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:Query"],
      resources: resourceArns,
    })
  );
  return getOwnerTokensLambda;
}
