#!/usr/bin/env node

import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { ECRDeploymentStack } from "../lib/cdk-ecr-stack";
import { config } from "dotenv";

config();

const app = new cdk.App();
new ECRDeploymentStack(app, "ECRDeploymentStack", {
  env: {
    account: "223605539824",
    region: "us-east-1",
  },
});
