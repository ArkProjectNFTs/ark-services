import * as lambda from "aws-cdk-lib/aws-lambda";
import * as iam from "aws-cdk-lib/aws-iam";
import * as cdk from "aws-cdk-lib";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { ArkStackProps } from "../../types";

export function getTokenLambda(scope: cdk.Stack, props: ArkStackProps) {
  const tableName = `ark_project_${props.envType}`;
  const getContractLambda = new lambda.Function(scope, "get-token", {
    code: lambda.Code.fromAsset("../../target/lambda/lambda-get-token"),
    runtime: lambda.Runtime.PROVIDED_AL2,
    handler: "not.required",
    environment: {
      RUST_BACKTRACE: "1",
      ARK_TABLE_NAME: tableName,
    },
    logRetention: RetentionDays.ONE_DAY,
  });
  getContractLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:GetItem"],
      resources: [
        `arn:aws:dynamodb:${scope.region}:${scope.account}:table/${tableName}`,
      ],
    })
  );
  return getContractLambda;
}
