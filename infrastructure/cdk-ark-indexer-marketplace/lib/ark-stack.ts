import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";

import { IndexerMarketplaceStackProps } from "./types";
import { deployIndexers } from "./ecs/indexer";
import { Vpc } from "aws-cdk-lib/aws-ec2";

export class IndexerMarketplaceStack extends cdk.Stack {
  constructor(
    scope: Construct,
    id: string,
    props: IndexerMarketplaceStackProps
  ) {
    super(scope, id, props);

    const vpc = Vpc.fromLookup(this, "ArkVPC", {
      vpcId: "vpc-0d11f7ec183208e08",
    });

    deployIndexers(
      this,
      props.networks,
      vpc,
      props.isProductionEnvironment,
      props.environmentName,
      props.indexerVersion
    );
  }
}
