#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { ArkRedisStack } from "../lib/cdk-ark-redis-stack";

const app = new cdk.App();
new ArkRedisStack(app, "ArkRedisStack", {
  env: {
    account: process.env.AWS_ACCOUNT_ID,
    region: process.env.AWS_REGION,
  },
  stackName: "ArkRedisStack",
});
