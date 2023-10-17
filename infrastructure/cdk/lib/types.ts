import * as cdk from "aws-cdk-lib";

export interface ArkStackProps extends cdk.StackProps {
  branch: string;
  stages: string[]
  isPullRequest: boolean;
}
