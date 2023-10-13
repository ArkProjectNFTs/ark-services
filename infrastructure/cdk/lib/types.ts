import * as cdk from "aws-cdk-lib";

export interface ArkStackProps extends cdk.StackProps {
  branch: 'main' | 'dev' | 'local';
  stages: string[]
}
