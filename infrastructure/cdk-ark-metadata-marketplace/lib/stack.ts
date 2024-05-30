import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";

import { MetadataMarketplaceIndexerStackProps } from "./types";
import { deployMetadataIndexer } from "./indexer";
import { Vpc } from "aws-cdk-lib/aws-ec2";

export class MetadataMarketplaceIndexerStack extends cdk.Stack {
  constructor(
    scope: Construct,
    id: string,
    props: MetadataMarketplaceIndexerStackProps
  ) {
    super(scope, id, props);

    const vpc = Vpc.fromLookup(this, "ArkVPC", {
      vpcId: "vpc-0d11f7ec183208e08",
    });

    deployMetadataIndexer(
      this,
      props.networks,
      vpc,
      props.isProductionEnvironment
    );
  }
}
