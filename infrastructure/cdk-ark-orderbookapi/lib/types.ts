import * as cdk from "aws-cdk-lib";

export interface ArkOrderbookApiStackProps extends cdk.StackProps {
  networks: string[];
  isProductionEnvironment: boolean;
  indexerVersion: string;
}
