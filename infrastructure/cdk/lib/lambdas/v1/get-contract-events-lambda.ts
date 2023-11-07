import * as cdk from 'aws-cdk-lib';
import { RustFunction } from 'cargo-lambda-cdk';
import * as iam from 'aws-cdk-lib/aws-iam';
import { RetentionDays } from 'aws-cdk-lib/aws-logs';
import { AssetHashType } from 'aws-cdk-lib';

export function getContractEventsLambda(scope: cdk.Stack, stages: string[]) {
  const getContractLambda = new RustFunction(scope, 'get-contract-events', {
    // Update the path to where your Rust project's Cargo.toml file is located
    manifestPath: '../../ark-lambdas/apigw/lambda-get-contract-events/Cargo.toml',
    environment: {
      RUST_BACKTRACE: "1",
    },
    logRetention: RetentionDays.ONE_DAY,
    bundling: {
      assetHashType: AssetHashType.OUTPUT,  // Set the assetHashType here
      // ...other bundling options if needed
    },
    // The bundling options are automatically handled by cargo-lambda-cdk.
    // If Cargo Lambda is installed locally, it will be used; otherwise, Docker will be used.
  });

  let resourceArns: string[] = [];

  for (const stage of stages) {
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}`
    );
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}/index/GSI1PK-GSI1SK-index`
    );
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/ark_project_${stage}_lambda_usage`
    );
  }

  getContractLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:Query", "dynamodb:PutItem"],
      resources: resourceArns,
    })
  );

  return getContractLambda;
}
