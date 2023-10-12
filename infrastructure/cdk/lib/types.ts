import * as cdk from "aws-cdk-lib";

export interface ArkStackProps extends cdk.StackProps {
  envType: 'mainnet' | 'testnet' | 'dev';
  branch: 'main' | 'dev' | 'local';
}
