#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { ArkProjectCdkIndexerStack } from "../lib/cdk-indexer-stack";
import * as dotenv from "dotenv";

dotenv.config();

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

console.log(`Context/Environment Configuration:`);
console.log(`------------------------------------`);
console.log(`- Branch: ${baseBranch}`);
console.log(`- Is a Release?: ${isRelease ? "Yes" : "No"}`);
console.log(`- Is a Pull Request?: ${isPullRequest ? "Yes" : "No"}`);
console.log(`- Target Branch: ${targetBranch}`);

if (isPullRequest) {
  console.log(`- Pull Request Number: ${prNumber}`);
}
console.log(`------------------------------------`);

if (isPullRequest && targetBranch && targetBranch !== "main") {
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
  stackNameSuffix = `PR${prNumber}`;
  console.log(`Deploying Pull Request stack for PR number: ${prNumber}`);
} else if (isRelease) {
  stackNameSuffix = "Production";
  console.log(`Deploying Production stack for release.`);
} else if (baseBranch === "main" && !isPullRequest) {
  stackNameSuffix = "Staging";
  console.log(`Deploying Staging stack.`);
}

console.log(`Determined stack name suffix: ${stackNameSuffix}`);

const stackName = `ArkIndexer${stackNameSuffix}`;

new ArkProjectCdkIndexerStack(app, stackName, {
  env: {
    account: process.env.AWS_ACCOUNT_ID,
    region: process.env.AWS_REGION,
  },
  indexerVersion: process.env.INDEXER_VERSION ?? "UNDEFINED",
  branch: baseBranch,
  isRelease: isRelease,
  isPullRequest: isPullRequest,
  prNumber: prNumber,
});
