import * as cdk from "aws-cdk-lib";
import { RustFunction } from "cargo-lambda-cdk";
import * as iam from "aws-cdk-lib/aws-iam";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { AssetHashType } from "aws-cdk-lib";
import { IVpc, SecurityGroup, SubnetType } from "aws-cdk-lib/aws-ec2";

export function getContractsLambda(
  scope: cdk.Stack,
  vpc: IVpc,
  lambdaSecurityGroup: SecurityGroup,
  stages: string[],
  tableNamePrefix: string
) {
  const indexName = "GSI2PK-GSI2SK-index";

  const getContractsLambda = new RustFunction(scope, "get-contracts", {
    manifestPath: "../../ark-lambdas/apigw/lambda-get-contracts/Cargo.toml",
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
  });

  let resourceArns: string[] = [];
  for (const stage of stages) {
    resourceArns.push(
      `arn:aws:dynamodb:${scope.region}:${scope.account}:table/${tableNamePrefix}_${stage}/index/${indexName}`
    );
  }

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

  return getContractsLambda;
}
