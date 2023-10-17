#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { ArkStack } from "../lib/ark-stack";
import { config } from "dotenv";

config();
const app = new cdk.App();

const branch = app.node.tryGetContext("branch") || process.env.BRANCH;
const isPullRequest =
  app.node.tryGetContext("isPullRequest") || process.env.IS_PULL_REQUEST;
const stages = ["mainnet", "testnet"];

const prNumber = app.node.tryGetContext("prNumber") || process.env.PR_NUMBER;

console.log('Branch:', branch);
console.log('Is Pull Request:', isPullRequest);
console.log('PR Number:', prNumber);

let stackNameSuffix;
if (isPullRequest === 'true') {
  stackNameSuffix = `pr-${prNumber}`;
} else {
  stackNameSuffix = branch === "main" ? "production" : "staging";
}

const stackName = `ArkStack-${stackNameSuffix}`;

new ArkStack(app, stackName, {
  env: {
    account: process.env.AWS_ACCOUNT_ID,
    region: process.env.AWS_REGION, // or whatever region you want to deploy to
  },
  branch: branch,
  stages: stages,
  isPullRequest: isPullRequest === 'true' ? true : false,
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
