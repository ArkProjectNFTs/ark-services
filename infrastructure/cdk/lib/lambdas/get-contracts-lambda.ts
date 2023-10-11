import * as lambda from "aws-cdk-lib/aws-lambda";
import * as iam from "aws-cdk-lib/aws-iam";
import * as cdk from "aws-cdk-lib";
import { RetentionDays } from "aws-cdk-lib/aws-logs";

export function getContractsLambda(scope: cdk.Stack) {
  const tableName = "ark_project_dev";
  const indexName = "GSI1PK-GSI1SK-index";
  const getContractLambda = new lambda.Function(scope, "get-contracts", {
    code: lambda.Code.fromAsset("../../target/lambda/lambda-get-contracts"),
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
      actions: ["dynamodb:Query"],
      resources: [
        `arn:aws:dynamodb:${scope.region}:${scope.account}:table/${tableName}/index/${indexName}`,
      ],
    })
  );
  return getContractLambda;
}
