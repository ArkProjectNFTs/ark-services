import * as cdk from "aws-cdk-lib";
import { RustFunction } from "cargo-lambda-cdk";
import * as iam from "aws-cdk-lib/aws-iam";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { AssetHashType } from "aws-cdk-lib";
import { IVpc, SecurityGroup, SubnetType } from "aws-cdk-lib/aws-ec2";

export function getOwnerEventsLambda(
  scope: cdk.Stack,
  vpc: IVpc,
  lambdaSecurityGroup: SecurityGroup,
  stages: string[],
  tableNamePrefix: string
) {
  const getOwnerEventsLambda = new RustFunction(scope, "get-owner-events", {
    manifestPath: "../../ark-lambdas/apigw/lambda-get-owner-events/Cargo.toml",
    environment: {
      RUST_BACKTRACE: "1",
    },
    logRetention: RetentionDays.ONE_DAY,
    bundling: {
      assetHashType: AssetHashType.OUTPUT,
    },
    vpc: vpc,
    vpcSubnets: {
      subnetType: SubnetType.PRIVATE_WITH_EGRESS,
    },
    securityGroups: [lambdaSecurityGroup],
  });

  let resourceArns: string[] = [];

  for (const stage of stages) {
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/${tableNamePrefix}_${stage}`
    );
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/${tableNamePrefix}_${stage}/index/GSI3PK-GSI3SK-index`,
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/${tableNamePrefix}_${stage}/index/GSI5PK-GSI5SK-index`
    );
  }

  getOwnerEventsLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:Query"],
      resources: resourceArns,
    })
  );

  return getOwnerEventsLambda;
}
