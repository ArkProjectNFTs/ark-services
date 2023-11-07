import * as cdk from "aws-cdk-lib";
import { RustFunction } from "cargo-lambda-cdk";
import * as iam from "aws-cdk-lib/aws-iam";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { AssetHashType } from "aws-cdk-lib";

export function getTokenLambda(scope: cdk.Stack, stages: string[]) {
  const getTokenLambda = new RustFunction(scope, "get-token", {
    manifestPath: "../../ark-lambdas/apigw/lambda-get-token/Cargo.toml",
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
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}`
    );
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}_lambda_usage`
    );
  }

  getTokenLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:GetItem", "dynamodb:PutItem"],
      resources: resourceArns,
    })
  );

  return getTokenLambda;
}
