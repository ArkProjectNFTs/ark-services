import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getContractTokensLambda } from "../../lambdas/v1/get-contract-tokens-lambda";
import { getTokenLambda } from "../../lambdas/v1/get-token-lambda";
import { postRefreshTokenMetadataLambda } from "../../lambdas/v1/post-refresh-token-metadata";
import * as cdk from "aws-cdk-lib";

export function tokensApi(
  scope: cdk.Stack,
  versionedRoot: apigateway.IResource,
  stages: string[],
  tableNamePrefix: string
) {
  const tokensResource = versionedRoot.addResource("tokens");
  const tokenContractAddressResource =
    tokensResource.addResource("{contract_address}");
  const tokensTokenIdResource =
    tokenContractAddressResource.addResource("{token_id}");

  // Get all tokens for a contract
  tokenContractAddressResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(
      getContractTokensLambda(scope, stages, tableNamePrefix),
      {
        proxy: true,
      }
    ),
    {
      apiKeyRequired: true, // API key is now required for this method
    }
  );

  // Get a specific token for a contract
  tokensTokenIdResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(
      getTokenLambda(scope, stages, tableNamePrefix),
      {
        proxy: true,
      }
    ),
    {
      apiKeyRequired: true, // API key is now required for this method
    }
  );

  const metadataRessource =
    tokenContractAddressResource.addResource("metadata");

  const refreshMetadataRessource = metadataRessource.addResource("refresh");

  refreshMetadataRessource.addMethod(
    "POST",
    new apigateway.LambdaIntegration(
      postRefreshTokenMetadataLambda(scope, stages, tableNamePrefix),
      {
        proxy: true,
      }
    ),
    {
      apiKeyRequired: true, // API key is now required for this method
    }
  );

  return versionedRoot;
}
