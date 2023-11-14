import * as cdk from "aws-cdk-lib";
import { RustFunction } from "cargo-lambda-cdk";
import * as iam from "aws-cdk-lib/aws-iam";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { AssetHashType } from "aws-cdk-lib";

export function getTokenEventsLambda(scope: cdk.Stack, stages: string[]) {
  const indexName = "GSI2PK-GSI2SK-index";
  const getTokenEventsLambda = new RustFunction(scope, "get-token-events", {
    manifestPath: "../../ark-lambdas/apigw/lambda-get-token-events/Cargo.toml",
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

  getTokenEventsLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:Query"],
      resources: resourceArns,
    })
  );

  return getTokenEventsLambda;
}
