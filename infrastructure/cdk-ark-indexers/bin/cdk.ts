#!/usr/bin/env node

import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { ArkIndexersStack } from "../lib/ark-stack";
import { config } from "dotenv";

config();
const app = new cdk.App();

const isProductionEnvironment: boolean =
  app.node.tryGetContext("isProductionEnvironment") === "true" ||
  process.env.DEPLOYMENT_ENV === "production";

const networks: string[] = ["mainnet", "sepolia"];

let environment = isProductionEnvironment ? "production" : "staging";
const stackName = `ark-indexers-${environment}`;
const indexerVersion: string = process.env.INDEXER_VERSION ?? "UNDEFINED";

new ArkIndexersStack(app, stackName, {
  env: {
    account: process.env.AWS_ACCOUNT_ID,
    region: process.env.AWS_REGION,
  },
  networks,
  isProductionEnvironment,
  indexerVersion,
});
