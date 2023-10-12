import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getOwnerTokensLambda } from "../lambdas/get-owner-tokens-lambda";
import * as cdk from "aws-cdk-lib";
import { ArkStackProps } from "../types";

export function ownerApi(
  scope: cdk.Stack,
  api: apigateway.RestApi,
  props: ArkStackProps
) {
  const ownerResource = api.root.addResource("owner");
  const ownerAddressResource = ownerResource.addResource("{owner_address}");

  // Get all tokens for an owner
  ownerAddressResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getOwnerTokensLambda(scope, props), { proxy: true })
  );
  return api;
}
