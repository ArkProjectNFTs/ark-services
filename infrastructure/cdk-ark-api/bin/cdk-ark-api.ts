#!/usr/bin/env node

import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { ArkApiStack } from "../lib/cdk-ark-api-stack";

const app = new cdk.App();

const stages: string[] = ["mainnet", "testnet"];
const isProductionEnvironment: boolean =
  app.node.tryGetContext("isProductionEnvironment") === "true" ||
  process.env.DEPLOYMENT_ENV === "production";

const environment = isProductionEnvironment ? "production" : "staging";

new ArkApiStack(app, `ark-api-${environment}`, {
  env: {
    account: "223605539824",
    region: "us-east-1",
  },
  stages: stages,
  isProductionEnvironment,
});
