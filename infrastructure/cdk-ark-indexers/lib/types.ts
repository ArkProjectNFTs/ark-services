import * as cdk from "aws-cdk-lib";

export interface ArkIndexersStackProps extends cdk.StackProps {
  networks: string[];
  isProductionEnvironment: boolean;
  indexerVersion: string;
}
