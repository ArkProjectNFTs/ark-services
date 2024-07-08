import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";

import { MarketplaceMetadataIndexerStackProps } from "./types";
import { deployMarketplaceMetadataIndexer } from "./indexer";
import { Vpc } from "aws-cdk-lib/aws-ec2";

export class MarketplaceMetadataIndexerStack extends cdk.Stack {
  constructor(
    scope: Construct,
    id: string,
    props: MarketplaceMetadataIndexerStackProps
  ) {
    super(scope, id, props);

    const vpc = Vpc.fromLookup(this, "ArkVPC", {
      vpcId: "vpc-0d11f7ec183208e08",
    });

    deployMarketplaceMetadataIndexer(
      this,
      props.networks,
      vpc,
      props.isProductionEnvironment
    );
  }
}
