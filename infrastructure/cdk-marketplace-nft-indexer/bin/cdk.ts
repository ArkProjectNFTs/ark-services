#!/usr/bin/env node

import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { MarketplaceNftIndexerStack } from "../lib/ark-stack";
import { config } from "dotenv";

config();
const app = new cdk.App();

const isProductionEnvironment: boolean =
  app.node.tryGetContext("isProductionEnvironment") === "true" ||
  process.env.DEPLOYMENT_ENV === "production";

const networks: string[] = ["mainnet"];

let environment = isProductionEnvironment ? "production" : "staging";
const stackName = `marketplace-nft-indexer-${environment}`;

new MarketplaceNftIndexerStack(app, stackName, {
  env: {
    account: process.env.AWS_ACCOUNT_ID,
    region: process.env.AWS_REGION,
  },
  networks,
  environmentName: environment,
  indexerVersion: process.env.INDEXER_VERSION,
});
