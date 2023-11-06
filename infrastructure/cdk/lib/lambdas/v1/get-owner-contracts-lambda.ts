import * as lambda from "aws-cdk-lib/aws-lambda";
import * as iam from "aws-cdk-lib/aws-iam";
import * as cdk from "aws-cdk-lib";
import { RetentionDays } from "aws-cdk-lib/aws-logs";

export function getOwnerContractsLambda(
  scope: cdk.Stack,
  stages: string[]
) {
  const indexName = "GSI2PK-GSI2SK-index";
  const getOwnerContractsLambda = new lambda.Function(scope, "get-owner-contracts", {
    code: lambda.Code.fromAsset("../../target/lambda/lambda-get-owner-contracts"),
    runtime: lambda.Runtime.PROVIDED_AL2,
    handler: "not.required",
    environment: {
      RUST_BACKTRACE: "1",
    },
    logRetention: RetentionDays.ONE_DAY,
  });

  let resourceArns: string[] = [];

  for (const stage of stages) {
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}/index/${indexName}`
    );
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}_lambda_usage`
    );
  }

  getOwnerContractsLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:Query", "dynamodb:PutItem"],
      resources: resourceArns,
    })
  );
  return getOwnerTokensLambda;
}
