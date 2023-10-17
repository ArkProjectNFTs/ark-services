import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getOwnerTokensLambda } from "../../lambdas/v1/get-owner-tokens-lambda";
import * as cdk from "aws-cdk-lib";

export function ownerApi(
  scope: cdk.Stack,
  versionedRoot: apigateway.IResource,
  stages: string[]
) {
  const ownerResource = versionedRoot.addResource("owners");
  const ownerAddressResource = ownerResource.addResource("{owner_address}");
  const ownerTokensRessource = ownerAddressResource.addResource("tokens");

  // Get all tokens for an owner
  ownerTokensRessource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getOwnerTokensLambda(scope, stages), { proxy: true })
  );
  return versionedRoot;
}
