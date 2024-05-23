#!/usr/bin/env node

import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { ArkDatabasesDeploymentStack } from "../lib/cdk-ark-databases";
import { config } from "dotenv";

config();

const app = new cdk.App();
new ArkDatabasesDeploymentStack(app, "ark-databases", {
  env: {
    account: process.env.AWS_ACCOUNT_ID,
    region: process.env.AWS_REGION,
  },
});
