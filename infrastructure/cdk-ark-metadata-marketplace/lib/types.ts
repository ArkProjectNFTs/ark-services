import * as cdk from "aws-cdk-lib";

export interface MetadataMarketplaceIndexerStackProps extends cdk.StackProps {
  networks: string[];
  isProductionEnvironment: boolean;
}
