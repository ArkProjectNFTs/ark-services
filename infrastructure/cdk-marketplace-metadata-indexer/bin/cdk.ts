#!/usr/bin/env node

import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { MarketplaceMetadataIndexerStack } from "../lib/stack";
import { config } from "dotenv";

config();
const app = new cdk.App();

const isProductionEnvironment: boolean =
  app.node.tryGetContext("isProductionEnvironment") === "true" ||
  process.env.DEPLOYMENT_ENV === "production";

const networks: string[] = ["mainnet"];

let environment = isProductionEnvironment ? "production" : "staging";
const stackName = `marketplace-metadata-indexer-${environment}`;

new MarketplaceMetadataIndexerStack(app, stackName, {
  env: {
    account: "223605539824",
    region: "us-east-1",
  },
  networks,
  isProductionEnvironment,
});
