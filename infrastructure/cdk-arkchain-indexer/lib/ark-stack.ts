import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";

import { ArkIndexersStackProps } from "./types";
import {
  Vpc,
  InstanceType,
  InstanceClass,
  InstanceSize,
} from "aws-cdk-lib/aws-ec2";
import {
  DatabaseInstance,
  DatabaseInstanceEngine,
  PostgresEngineVersion,
} from "aws-cdk-lib/aws-rds";
import { SecretValue } from "aws-cdk-lib";
import { deployIndexer } from "./ecs/indexer";

export class ArkIndexersStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: ArkIndexersStackProps) {
    super(scope, id, props);

    const vpc = Vpc.fromLookup(this, "ArkVPC", {
      vpcId: process.env.VPC_ID,
    });

    const dbSecurityGroup = new cdk.aws_ec2.SecurityGroup(
      this,
      "DBSecurityGroup",
      {
        vpc,
        description: "Allow access to RDS from ECS",
      }
    );

    dbSecurityGroup.addIngressRule(
      cdk.aws_ec2.Peer.anyIpv4(),
      cdk.aws_ec2.Port.tcp(5432),
      "Allow inbound PostgreSQL traffic"
    );

    const network = props.isProductionEnvironment ? "production" : "staging";
    // create postgres database
    const dbInstance = new DatabaseInstance(
      this,
      `arkchain-indexer-${network}-db`,
      {
        engine: DatabaseInstanceEngine.postgres({
          version: PostgresEngineVersion.VER_15_4,
        }),
        parameters: {
          'rds.force_ssl': '0',
        },
        instanceType: InstanceType.of(InstanceClass.T3, InstanceSize.MICRO),
        vpc,
        allocatedStorage: 20,
        maxAllocatedStorage: 100,
        deleteAutomatedBackups: true,
        backupRetention: cdk.Duration.days(0),
        deletionProtection: false,
        databaseName: "arkchainindexer",
        credentials: {
          username: process.env.DB_USERNAME || "defaultUsername",
          password: SecretValue.unsafePlainText(
            process.env.DB_PASSWORD || "defaultPassword"
          ),
        },
        securityGroups: [dbSecurityGroup],
      }
    );

    dbInstance.connections.allowFrom(
      dbSecurityGroup,
      cdk.aws_ec2.Port.tcp(5432)
    );

    const dbEndpointAddress = dbInstance.dbInstanceEndpointAddress;

    return;
    deployIndexer(
      this,
      props.networks,
      props.isProductionEnvironment,
      vpc,
      dbEndpointAddress,
    );
  }
}
