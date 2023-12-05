import * as cdk from "aws-cdk-lib";

export interface ArkStackProps extends cdk.StackProps {
  stages: string[];
  isRelease: boolean;
  indexerVersion: string;
}
