#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { ArkStack } from "../lib/ark-stack";
import { config } from "dotenv";

config();
const app = new cdk.App();

const envType = app.node.tryGetContext("envType") || process.env.ENV_TYPE;
const branch = app.node.tryGetContext("branch") || process.env.BRANCH;
const stageName = branch === "main" ? "prod" : "staging";

new ArkStack(app, `ArkStack-${envType}-${stageName}`, {
  env: {
    account: process.env.AWS_ACCOUNT_ID,
    region: process.env.AWS_REGION  // or whatever region you want to deploy to
  },
  envType: envType,
  branch: branch,
  description:
    "This stack provisions the infrastructure for the Ark Project, which includes API endpoints for contract management and token events. It integrates with DynamoDB for data storage and provides Lambda functions for specific API operations. The stack is designed to be environment-agnostic and can be deployed to any AWS region.",
  /* If you don't specify 'env', this stack will be environment-agnostic.
   * Account/Region-dependent features and context lookups will not work,
   * but a single synthesized template can be deployed anywhere. */

  /* Uncomment the next line to specialize this stack for the AWS Account
   * and Region that are implied by the current CLI configuration. */
  // env: { account: process.env.CDK_DEFAULT_ACCOUNT, region: process.env.CDK_DEFAULT_REGION },

  /* Uncomment the next line if you know exactly what Account and Region you
   * want to deploy the stack to. */
  // env: { account: '123456789012', region: 'us-east-1' },

  /* For more information, see https://docs.aws.amazon.com/cdk/latest/guide/environments.html */
});
