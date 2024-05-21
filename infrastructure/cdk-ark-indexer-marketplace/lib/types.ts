import * as cdk from "aws-cdk-lib";

export interface IndexerMarketplaceStackProps extends cdk.StackProps {
  networks: string[];
  isProductionEnvironment: boolean;
  indexerVersion: string;
}
