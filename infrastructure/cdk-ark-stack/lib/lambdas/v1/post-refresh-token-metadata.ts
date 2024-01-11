import * as cdk from "aws-cdk-lib";
import { RustFunction } from "cargo-lambda-cdk";
import * as iam from "aws-cdk-lib/aws-iam";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { AssetHashType } from "aws-cdk-lib";
import { IVpc, SecurityGroup, SubnetType } from "aws-cdk-lib/aws-ec2";

export function postRefreshTokenMetadataLambda(
  scope: cdk.Stack,
  vpc: IVpc,
  lambdaSecurityGroup: SecurityGroup,
  stages: string[],
  tableNamePrefix: string
) {
  const postRefreshTokenMetadataLambda = new RustFunction(
    scope,
    "post-refresh-token-metadata",
    {
      manifestPath:
        "../../ark-lambdas/apigw/lambda-post-refresh-token-metadata/Cargo.toml",
      environment: {
        RUST_BACKTRACE: "1",
      },
      bundling: {
        assetHashType: AssetHashType.OUTPUT,
      },
      logRetention: RetentionDays.ONE_DAY,
      vpc: vpc,
      vpcSubnets: {
        subnetType: SubnetType.PRIVATE_WITH_EGRESS,
      },
      securityGroups: [lambdaSecurityGroup],
    }
  );

  let resourceArns: string[] = [];

  for (const stage of stages) {
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/${tableNamePrefix}_${stage}`
    );
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/${tableNamePrefix}_${stage}_lambda_usage`
    );
  }

  postRefreshTokenMetadataLambda.addToRolePolicy(
    new iam.PolicyStatement({
      actions: ["dynamodb:GetItem", "dynamodb:PutItem", "dynamodb:UpdateItem"],
      resources: resourceArns,
    })
  );

  return postRefreshTokenMetadataLambda;
}
