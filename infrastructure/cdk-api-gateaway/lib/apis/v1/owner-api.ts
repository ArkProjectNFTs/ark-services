import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getOwnerTokensLambda } from "../../lambdas/v1/get-owner-tokens-lambda";
import { getOwnerContractsLambda } from "../../lambdas/v1/get-owner-contracts-lambda";
import * as cdk from "aws-cdk-lib";
import { getOwnerEventsLambda } from "../../lambdas/v1/get-owner-events-lambda";

export function ownerApi(
  scope: cdk.Stack,
  versionedRoot: apigateway.IResource,
  stages: string[]
) {
  const ownerResource = versionedRoot.addResource("owners");
  const ownerAddressResource = ownerResource.addResource("{owner_address}");
  const ownerTokensRessource = ownerAddressResource.addResource("tokens");
  const ownerEventsRessource = ownerAddressResource.addResource("events");
  const ownerContractsRessource = ownerAddressResource.addResource("contracts");

  // Get all tokens for an owner
  ownerTokensRessource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getOwnerTokensLambda(scope, stages), {
      proxy: true,
    }),
    {
      apiKeyRequired: true, // API key is now required for this method
    }
  );

  // Get all contracts for an owner
  ownerContractsRessource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getOwnerContractsLambda(scope, stages), {
      proxy: true,
    }),
    {
      apiKeyRequired: true, // API key is now required for this method
    }
  );

  // Get all event for an owner
  ownerEventsRessource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getOwnerEventsLambda(scope, stages), {
      proxy: true,
    }),
    {
      apiKeyRequired: true, // API key is now required for this method
    }
  );

  return versionedRoot;
}
