import * as cdk from "aws-cdk-lib";

export interface MarketplaceMetadataIndexerStackProps extends cdk.StackProps {
  networks: string[];
  isProductionEnvironment: boolean;
}
