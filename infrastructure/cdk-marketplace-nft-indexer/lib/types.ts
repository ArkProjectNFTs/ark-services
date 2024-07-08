import * as cdk from "aws-cdk-lib";

export interface MarketplaceNftIndexerStackProps extends cdk.StackProps {
  networks: string[];
  environmentName: string;
  indexerVersion?: string;
}
