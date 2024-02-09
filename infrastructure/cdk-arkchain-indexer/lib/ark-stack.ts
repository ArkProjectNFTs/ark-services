import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";

import { ArkIndexersStackProps } from "./types";
import { Vpc, InstanceType, InstanceClass, InstanceSize } from "aws-cdk-lib/aws-ec2";
import {
  DatabaseInstance,
  DatabaseInstanceEngine,
  PostgresEngineVersion,
} from 'aws-cdk-lib/aws-rds';
import { SecretValue } from "aws-cdk-lib";
import { deployIndexer } from "./ecs/indexer";

export class ArkIndexersStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: ArkIndexersStackProps) {
    super(scope, id, props);

    const vpc = Vpc.fromLookup(this, "ArkVPC", {
      vpcId: "vpc-0d11f7ec183208e08",
    });

    // create postgres database
    new DatabaseInstance(this, 'arkchain-indexer-staging-dbpgarkchainc6c1584f-er1cnatb6f8y', {
      engine: DatabaseInstanceEngine.postgres({
        version: PostgresEngineVersion.VER_15_4,
      }),
      instanceType: InstanceType.of(InstanceClass.T3, InstanceSize.MICRO),
      vpc,
      allocatedStorage: 20,
      maxAllocatedStorage: 100,
      deleteAutomatedBackups: true,
      backupRetention: cdk.Duration.days(0),
      deletionProtection: false,
      databaseName: 'arkchainindexer',
      credentials: {
        username: process.env.DB_USERNAME || 'defaultUsername',
        password: SecretValue.unsafePlainText(process.env.DB_PASSWORD || 'defaultPassword'),
      },
    });

    deployIndexer(
      this,
      props.networks,
      props.isProductionEnvironment,
      props.indexerVersion,
      vpc
    );

  }
}
