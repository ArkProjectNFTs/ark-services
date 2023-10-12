import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getContractTokensLambda } from "../../lambdas/v1/get-contract-tokens-lambda";
import { getTokenLambda } from "../../lambdas/v1/get-token-lambda";
import * as cdk from "aws-cdk-lib";
import { ArkStackProps } from "../../types";

export function tokensApi(
  scope: cdk.Stack,
  versionedRoot: apigateway.IResource,
  props: ArkStackProps
) {
  const tokensResource = versionedRoot.addResource("tokens");
  const tokenContractAddressResource =
    tokensResource.addResource("{contract_address}");
  const tokensTokenIdResource =
    tokenContractAddressResource.addResource("{token_id}");

  // Get all tokens for a contract
  tokenContractAddressResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getContractTokensLambda(scope, props), {
      proxy: true,
    })
  );

  // Get a specific token for a contract
  tokensTokenIdResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getTokenLambda(scope, props), {
      proxy: true,
    })
  );
  return versionedRoot;
}
