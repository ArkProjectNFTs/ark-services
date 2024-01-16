import * as cdk from "aws-cdk-lib";

export interface ArkApiStackProps extends cdk.StackProps {
  stages: string[];
  isProductionEnvironment: boolean;
}
