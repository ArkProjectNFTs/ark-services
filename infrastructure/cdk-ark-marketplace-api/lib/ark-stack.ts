import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";

import { ArkMarketplaceApiStackProps } from "./types";
import { Vpc } from "aws-cdk-lib/aws-ec2";
import { deployApi } from "./ecs/api";

export class ArkMarketplaceApiStack extends cdk.Stack {
  constructor(
    scope: Construct,
    id: string,
    props: ArkMarketplaceApiStackProps
  ) {
    super(scope, id, props);

    const vpc = Vpc.fromLookup(this, "ArkVPC", {
      vpcId: process.env.VPC_ID,
    });

    deployApi(this, props.networks, props.isProductionEnvironment, vpc);
  }
}
