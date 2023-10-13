import * as lambda from "aws-cdk-lib/aws-lambda";
import * as iam from "aws-cdk-lib/aws-iam";
import * as cdk from "aws-cdk-lib";
import { RetentionDays } from "aws-cdk-lib/aws-logs";

export function getContractEventsLambda(scope: cdk.Stack, stages: string[]) {
  const getContractLambda = new lambda.Function(scope, "get-contract-events", {
    code: lambda.Code.fromAsset(
      "../../target/lambda/lambda-get-contract-events"
    ),
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
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}/index/GSI1PK-GSI1SK-index`
    );
  }

  getContractLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:Query"],
      resources: resourceArns,
    })
  );
  return getContractLambda;
}
