import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getOwnerTokensLambda } from "../lambdas/get-owner-tokens-lambda";
import * as cdk from "aws-cdk-lib";

export function ownerApi(scope: cdk.Stack, api: apigateway.RestApi) {
  const lambda = getOwnerTokensLambda(scope);
  const ownerResource = api.root.addResource("owner");
  const ownerAddressResource = ownerResource.addResource("{owner_address}");

  // Get all tokens for an owner
  ownerAddressResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(lambda, { proxy: true })
  );
  return api;
}
