#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { ArkRedisStack } from "../lib/cdk-ark-redis-stack";

const app = new cdk.App();

// Explicit boolean type checking and conversion
const isProductionEnvironment: boolean =
  app.node.tryGetContext("isProductionEnvironment") === "true" ||
  process.env.DEPLOYMENT_ENV === "production";

const stackName = `ark-redis-${
  isProductionEnvironment ? "production" : "staging"
}`;

new ArkRedisStack(app, stackName, {
  env: {
    account: "223605539824",
    region: "us-east-1",
  },
  isProductionEnvironment,
  stackName,
});
