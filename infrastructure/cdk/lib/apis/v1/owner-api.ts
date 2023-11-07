import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getOwnerTokensLambda } from "../../lambdas/v1/get-owner-tokens-lambda";
import { getOwnerContractsLambda } from "../../lambdas/v1/get-owner-contracts-lambda";
import * as cdk from "aws-cdk-lib";

export function ownerApi(
  scope: cdk.Stack,
  versionedRoot: apigateway.IResource,
  stages: string[]
) {
  const ownerResource = versionedRoot.addResource("owners");
  const ownerAddressResource = ownerResource.addResource("{owner_address}");
  const ownerTokensRessource = ownerAddressResource.addResource("tokens");
  const ownerContractsRessource = ownerAddressResource.addResource("contracts");

  // Get all tokens for an owner
  ownerTokensRessource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getOwnerTokensLambda(scope, stages), { proxy: true })
  );

  // Get all contracts for an owner
  ownerContractsRessource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getOwnerContractsLambda(scope, stages), { proxy: true })
  );

  return versionedRoot;
}
