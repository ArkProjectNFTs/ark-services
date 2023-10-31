import * as lambda from "aws-cdk-lib/aws-lambda";
import * as iam from "aws-cdk-lib/aws-iam";
import * as cdk from "aws-cdk-lib";
import { RetentionDays } from "aws-cdk-lib/aws-logs";

export function getContractLambda(scope: cdk.Stack, stages: string[]) {
  const getContractLambda = new lambda.Function(scope, "get-contract", {
    code: lambda.Code.fromAsset("../../target/lambda/lambda-get-contract"),
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
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}`
    );
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}_lambda_usage`
    );
  }

  getContractLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:GetItem"],
      resources: resourceArns,
    })
  );
  return getContractLambda;
}
