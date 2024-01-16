import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";

import { ArkIndexersStackProps } from "./types";
import { deployIndexer } from "./ecs/indexer";
import { Vpc } from "aws-cdk-lib/aws-ec2";

export class ArkIndexersStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: ArkIndexersStackProps) {
    super(scope, id, props);

    const vpc = Vpc.fromLookup(this, "ArkVPC", {
      vpcId: "vpc-0d11f7ec183208e08",
    });

    deployIndexer(
      this,
      props.networks,
      vpc,
      props.isProductionEnvironment,
      props.indexerVersion
    );
  }
}
