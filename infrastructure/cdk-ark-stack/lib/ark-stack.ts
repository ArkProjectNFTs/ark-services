import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";

import { ArkStackProps } from "./types";
import { deployIndexer } from "./ecs/indexer";
import { Vpc } from "aws-cdk-lib/aws-ec2";

export class ArkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: ArkStackProps) {
    super(scope, id, props);

    const vpc = Vpc.fromLookup(this, "ArkVPC", {
      vpcId: "vpc-0d11f7ec183208e08",
    });

    deployIndexer(
      this,
      vpc,
      props.isProductionEnvironment,
      props.indexerVersion
    );
  }
}
