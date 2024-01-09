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
  description:
    "This stack provisions the infrastructure for the Ark Project, which includes API endpoints for contract management and token events. It integrates with DynamoDB for data storage and provides Lambda functions for specific API operations. The stack is designed to be environment-agnostic and can be deployed to any AWS region.",
  stackName: "ArkRedisStack",
});
