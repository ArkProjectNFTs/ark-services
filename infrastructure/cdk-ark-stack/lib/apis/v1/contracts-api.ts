import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getContractsLambda } from "../../lambdas/v1/get-contracts-lambda";
import { getContractLambda } from "../../lambdas/v1/get-contract-lambda";
import * as cdk from "aws-cdk-lib";

export function contractsApi(
  scope: cdk.Stack,
  versionedRoot: apigateway.IResource,
  stages: string[]
) {
  const contracts = versionedRoot.addResource("contracts");
  const contractsContractAddressRessource =
    contracts.addResource("{contract_address}");

  // Get all contracts
  contracts.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getContractsLambda(scope, stages), {
      proxy: true,
    }),
    {
      apiKeyRequired: true, // API key is now required for this method
    }
  );

  // Get a specific contract
  contractsContractAddressRessource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getContractLambda(scope, stages), {
      proxy: true,
    }),
    {
      apiKeyRequired: true, // API key is now required for this method
    }
  );
  return versionedRoot;
}
