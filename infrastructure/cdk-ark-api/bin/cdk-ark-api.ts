#!/usr/bin/env node

import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { ArkApiStack } from "../lib/cdk-ark-api-stack";

const app = new cdk.App();

const stages: string[] = ["mainnet", "testnet"];
const isProductionEnvironment: boolean =
  app.node.tryGetContext("isProductionEnvironment") === "true" ||
  process.env.DEPLOYMENT_ENV === "production";

const stackNameSuffix = isProductionEnvironment ? "production" : "staging";

new ArkApiStack(app, `ArkApiStack-${stackNameSuffix}`, {
  env: {
    account: "223605539824",
    region: "us-east-1",
  },
  stages: stages,
  isProductionEnvironment,
  description:
    "This stack provisions the infrastructure for the Ark Project, which includes API endpoints for contract management and token events. It integrates with DynamoDB for data storage and provides Lambda functions for specific API operations. The stack is designed to be environment-agnostic and can be deployed to any AWS region.",
});
