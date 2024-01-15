#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { ArkIndexersStack } from "../lib/ark-stack";
import { config } from "dotenv";

config();
const app = new cdk.App();

// Explicit boolean type checking and conversion
const isProductionEnvironment: boolean =
  app.node.tryGetContext("isProductionEnvironment") === "true" ||
  process.env.DEPLOYMENT_ENV === "production";

const networks: string[] = isProductionEnvironment
  ? ["mainnet", "testnet"]
  : ["mainnet"];

let stackNameSuffix;
if (isProductionEnvironment) {
  stackNameSuffix = "production";
  console.log(`Deploying Production stack for release.`);
} else {
  stackNameSuffix = "staging";
  console.log(`Deploying Staging stack.`);
}

console.log(`Determined stack name suffix: ${stackNameSuffix}`);
const stackName = `ArkIndexersStack-${stackNameSuffix}`;
const indexerVersion: string = process.env.INDEXER_VERSION ?? "UNDEFINED";

new ArkIndexersStack(app, stackName, {
  env: {
    account: process.env.AWS_ACCOUNT_ID,
    region: process.env.AWS_REGION,
  },
  networks,
  isProductionEnvironment,
  indexerVersion,
  description:
    "This stack provisions the infrastructure for the Ark Project, which includes API endpoints for contract management and token events. It integrates with DynamoDB for data storage and provides Lambda functions for specific API operations. The stack is designed to be environment-agnostic and can be deployed to any AWS region.",
});
