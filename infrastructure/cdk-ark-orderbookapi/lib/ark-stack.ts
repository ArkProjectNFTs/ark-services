import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";

import { ArkOrderbookApiStackProps } from "./types";
import {
  Vpc,
} from "aws-cdk-lib/aws-ec2";
import { deployApi } from "./ecs/api";

export class ArkIndexersStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: ArkOrderbookApiStackProps) {
    super(scope, id, props);

    const vpc = Vpc.fromLookup(this, "ArkVPC", {
      vpcId: process.env.VPC_ID,
    });

    deployApi(
      this,
      props.networks,
      props.isProductionEnvironment,
      vpc,
    );
  }
}
