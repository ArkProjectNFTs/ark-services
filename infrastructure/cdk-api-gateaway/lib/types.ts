import * as cdk from "aws-cdk-lib";

export interface ArkStackProps extends cdk.StackProps {
  branch: string;
  stages: string[]
  isRelease: boolean;
  isPullRequest: boolean;
  prNumber: string;
}
