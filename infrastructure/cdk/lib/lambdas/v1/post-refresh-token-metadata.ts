import * as lambda from "aws-cdk-lib/aws-lambda";
import * as iam from "aws-cdk-lib/aws-iam";
import * as cdk from "aws-cdk-lib";
import { RetentionDays } from "aws-cdk-lib/aws-logs";

export function postRefreshTokenMetadataLambda(
  scope: cdk.Stack,
  stages: string[]
) {
  const postRefreshTokenMetadata = new lambda.Function(
    scope,
    "post-refresh-token-metadata",
    {
      code: lambda.Code.fromAsset(
        "../../target/lambda/lambda-post-refresh-token-metadata"
      ),
      runtime: lambda.Runtime.PROVIDED_AL2,
      handler: "not.required",
      environment: {
        RUST_BACKTRACE: "1",
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

  postRefreshTokenMetadata.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:GetItem", "dynamodb:PutItem"],
      resources: resourceArns,
    })
  );
  return postRefreshTokenMetadata;
}
