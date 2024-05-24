#!/usr/bin/env node

import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { ArkMarketplaceApiStack } from "../lib/ark-stack";
import { config } from "dotenv";

config();
const app = new cdk.App();

const isProductionEnvironment: boolean =
  app.node.tryGetContext("isProductionEnvironment") === "true" ||
  process.env.DEPLOYMENT_ENV === "production";

const networks: string[] = isProductionEnvironment ? ["mainnet"] : ["mainnet"];
let environment = isProductionEnvironment ? "production" : "staging";

new ArkMarketplaceApiStack(app, `ark-marketplace-api-${environment}`, {
  env: {
    account: process.env.AWS_ACCOUNT_ID,
    region: process.env.AWS_REGION,
  },
  networks,
  isProductionEnvironment,
});
