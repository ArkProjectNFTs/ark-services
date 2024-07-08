import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";

import { MarketplaceNftIndexerStackProps } from "./types";
import { deployMarketplaceNftIndexer } from "./ecs/indexer";
import { Vpc } from "aws-cdk-lib/aws-ec2";

export class MarketplaceNftIndexerStack extends cdk.Stack {
  constructor(
    scope: Construct,
    id: string,
    props: MarketplaceNftIndexerStackProps
  ) {
    super(scope, id, props);

    const vpc = Vpc.fromLookup(this, "ArkVPC", {
      vpcId: "vpc-0d11f7ec183208e08",
    });

    deployMarketplaceNftIndexer(
      this,
      props.networks,
      vpc,
      props.environmentName,
      props.indexerVersion
    );
  }
}
