#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { ArkStack } from "../lib/ark-stack";
import { config } from "dotenv";

config();
const app = new cdk.App();

const baseBranch: string =
  app.node.tryGetContext("branch") || process.env.BRANCH;
const targetBranch: string = app.node.tryGetContext("targetBranch");

// Explicit boolean type checking and conversion
const isRelease: boolean =
  app.node.tryGetContext("isRelease") === "true" ||
  process.env.IS_RELEASE === "true";
const isPullRequest: boolean =
  app.node.tryGetContext("isPullRequest") === "true" ||
  process.env.IS_PULL_REQUEST === "true";
const prNumber: string =
  app.node.tryGetContext("prNumber") || process.env.PR_NUMBER;
const stages: string[] = ["mainnet", "testnet"];

console.log(`Context/Environment Configuration:`);
console.log(`------------------------------------`);
console.log(`- Branch: ${baseBranch}`);
console.log(`- Is a Release?: ${isRelease ? "Yes" : "No"}`);
console.log(`- Is a Pull Request?: ${isPullRequest ? "Yes" : "No"}`);
if (isPullRequest) {
  console.log(`- Pull Request Number: ${prNumber}`);
}
console.log(`------------------------------------`);

if (isPullRequest && (!targetBranch || targetBranch !== "main")) {
  throw new Error(
    "Pull Requests can only target the 'main' branch for deployments."
  );
}

if (!isPullRequest && !isRelease && baseBranch !== "main") {
  throw new Error(
    `Deployments are only allowed for main, releases, and pull requests. The branch ${baseBranch} does not meet these criteria.`
  );
}

let stackNameSuffix: string = "default";
if (isPullRequest) {
  stackNameSuffix = `pr-${prNumber}`;
  console.log(`Deploying Pull Request stack for PR number: ${prNumber}`);
} else if (isRelease) {
  stackNameSuffix = "production";
  console.log(`Deploying Production stack for release.`);
} else if (baseBranch === "main" && !isPullRequest) {
  stackNameSuffix = "staging";
  console.log(`Deploying Staging stack.`);
}

console.log(`Determined stack name suffix: ${stackNameSuffix}`);

const stackName = `ArkStack-${stackNameSuffix}`;

new ArkStack(app, stackName, {
  env: {
    account: process.env.AWS_ACCOUNT_ID,
    region: process.env.AWS_REGION, // or whatever region you want to deploy to
  },
  branch: baseBranch,
  stages: stages,
  isRelease: isRelease,
  isPullRequest: isPullRequest,
  prNumber: prNumber,
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
