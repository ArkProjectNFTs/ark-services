import * as cdk from "aws-cdk-lib";

export interface ArkMarketplaceApiStackProps extends cdk.StackProps {
  networks: string[];
  isProductionEnvironment: boolean;
}
