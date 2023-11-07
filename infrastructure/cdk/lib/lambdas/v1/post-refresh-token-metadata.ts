import * as cdk from "aws-cdk-lib";
import { RustFunction } from "cargo-lambda-cdk";
import * as iam from "aws-cdk-lib/aws-iam";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { AssetHashType } from "aws-cdk-lib";

export function postRefreshTokenMetadataLambda(
  scope: cdk.Stack,
  stages: string[]
) {
  const postRefreshTokenMetadataLambda = new RustFunction(
    scope,
    "post-refresh-token-metadata",
    {
      manifestPath:
        "../../ark-lambdas/apigw/lambda-post-refresh-token-metadata/Cargo.toml",
      environment: {
        RUST_BACKTRACE: "1",
      },
      bundling: {
        assetHashType: AssetHashType.SOURCE, // Set the assetHashType here
        // ...other bundling options if needed
      },
      logRetention: RetentionDays.ONE_DAY,
    }
  );

  let resourceArns: string[] = [];

  for (const stage of stages) {
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}`
    );
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}_lambda_usage`
    );
  }

  postRefreshTokenMetadataLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:GetItem", "dynamodb:PutItem"],
      resources: resourceArns,
    })
  );

  return postRefreshTokenMetadataLambda;
}
