import * as cdk from "aws-cdk-lib";
import { RustFunction } from "cargo-lambda-cdk";
import * as iam from "aws-cdk-lib/aws-iam";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { AssetHashType } from "aws-cdk-lib";
import { IVpc, SubnetType } from "aws-cdk-lib/aws-ec2";

export function getContractsLambda(
  scope: cdk.Stack,
  vpc: IVpc,
  stages: string[],
  tableNamePrefix: string
) {
  const lambdaSubnets = {
    subnetType: SubnetType.PRIVATE_WITH_EGRESS,
  };

  const indexName = "GSI2PK-GSI2SK-index";
  // Define a RustFunction using the cargo-lambda-cdk construct
  const getContractsLambda = new RustFunction(scope, "get-contracts", {
    // Specify the path to your Rust project's Cargo.toml file
    manifestPath: "../../ark-lambdas/apigw/lambda-get-contracts/Cargo.toml",
    environment: {
      RUST_BACKTRACE: "1",
    },
    bundling: {
      assetHashType: AssetHashType.OUTPUT, // Set the assetHashType here
      // ...other bundling options if needed
    },
    logRetention: RetentionDays.ONE_DAY,
    // Additional bundling options can be specified if necessary
    vpc: vpc, // Set the VPC
    vpcSubnets: lambdaSubnets, // Set the subnets
  });

  let resourceArns: string[] = [];

  // Construct the necessary resource ARNs from the provided stages
  for (const stage of stages) {
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/${tableNamePrefix}_${stage}/index/${indexName}`
    );
  }

  // Add permissions to the Lambda's role to interact with DynamoDB
  getContractsLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:Query"],
      resources: resourceArns,
    })
  );

  getContractsLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["elasticache:*"],
      resources: [
        "arn:aws:elasticache:us-east-1:223605539824:cluster:prodrediscluster",
      ],
    })
  );

  // Return the RustFunction construct
  return getContractsLambda;
}
