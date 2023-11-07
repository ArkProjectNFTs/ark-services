import * as cdk from 'aws-cdk-lib';
import { RustFunction } from 'cargo-lambda-cdk';
import * as iam from 'aws-cdk-lib/aws-iam';
import { RetentionDays } from 'aws-cdk-lib/aws-logs';

export function getContractLambda(scope: cdk.Stack, stages: string[]) {
  // Define a RustFunction using the cargo-lambda-cdk construct
  const getContractLambda = new RustFunction(scope, 'get-contract', {
    // Specify the path to your Rust project's Cargo.toml file
    manifestPath: '../../ark-lambdas/apigw/lambda-get-contract/Cargo.toml',
    environment: {
      RUST_BACKTRACE: "1",
    },
    logRetention: RetentionDays.ONE_DAY,
    // Additional bundling options can be specified if necessary
  });

  let resourceArns: string[] = [];

  // Construct the necessary resource ARNs from the provided stages
  for (const stage of stages) {
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}`
    );
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}_lambda_usage`
    );
  }

  // Add permissions to the Lambda's role to interact with DynamoDB
  getContractLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:GetItem", "dynamodb:PutItem"],
      resources: resourceArns,
    })
  );

  // Return the RustFunction construct
  return getContractLambda;
}
