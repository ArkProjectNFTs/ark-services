import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getContractTokensLambda } from "../lambdas/get-contract-tokens-lambda";
import { getTokenLambda } from "../lambdas/get-token-lambda";
import * as cdk from "aws-cdk-lib";

export function tokensApi(
  scope: cdk.Stack,
  api: apigateway.RestApi
) {
  const tokensResource = api.root.addResource("tokens");
  const tokenContractAddressResource =
    tokensResource.addResource("{contract_address}");
  const tokensTokenIdResource =
    tokenContractAddressResource.addResource("{token_id}");

  // Get all tokens for a contract
  tokenContractAddressResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getContractTokensLambda(scope), {
      proxy: true,
    })
  );

  // Get a specific token for a contract
  tokensTokenIdResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getTokenLambda(scope), {
      proxy: true,
    })
  );
  return api;
}
