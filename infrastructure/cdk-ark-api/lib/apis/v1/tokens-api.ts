import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { postBatchTokensLambda } from "../../lambdas/v1/post-batch-tokens-lambda";
import { getContractTokensLambda } from "../../lambdas/v1/get-contract-tokens-lambda";
import { getTokenLambda } from "../../lambdas/v1/get-token-lambda";
import { postRefreshTokenMetadataLambda } from "../../lambdas/v1/post-refresh-token-metadata";
import * as cdk from "aws-cdk-lib";
import { ISecurityGroup, IVpc } from "aws-cdk-lib/aws-ec2";

export function tokensApi(
  scope: cdk.Stack,
  vpc: IVpc,
  lambdaSecurityGroup: ISecurityGroup,
  versionedRoot: apigateway.IResource,
  stages: string[],
  tableNamePrefix: string
) {
  const tokensResource = versionedRoot.addResource("tokens");

  const tokensBatchResource = tokensResource.addResource("batch");
  tokensBatchResource.addMethod(
    "POST",
    new apigateway.LambdaIntegration(
      postBatchTokensLambda(
        scope,
        vpc,
        lambdaSecurityGroup,
        stages,
        tableNamePrefix
      ),
      {
        proxy: true,
      }
    ),
    {
      apiKeyRequired: true,
    }
  );

  const tokenContractAddressResource =
    tokensResource.addResource("{contract_address}");

  const tokensTokenIdResource =
    tokenContractAddressResource.addResource("{token_id}");

  // Get all tokens for a contract
  tokenContractAddressResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(
      getContractTokensLambda(
        scope,
        vpc,
        lambdaSecurityGroup,
        stages,
        tableNamePrefix
      ),
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
      getTokenLambda(scope, vpc, lambdaSecurityGroup, stages, tableNamePrefix),
      {
        proxy: true,
      }
    ),
    {
      apiKeyRequired: true, // API key is now required for this method
    }
  );

  const metadataRessource = tokensTokenIdResource.addResource("metadata");
  const refreshMetadataRessource = metadataRessource.addResource("refresh");

  refreshMetadataRessource.addMethod(
    "POST",
    new apigateway.LambdaIntegration(
      postRefreshTokenMetadataLambda(
        scope,
        vpc,
        lambdaSecurityGroup,
        stages,
        tableNamePrefix
      ),
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
