import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getContractsLambda } from "../lambdas/get-contracts-lambda";
import { getContractLambda } from "../lambdas/get-contract-lambda";
import * as cdk from "aws-cdk-lib";

export function contractsApi(scope: cdk.Stack, api: apigateway.RestApi) {
  const contracts = api.root.addResource("contracts");
  const contractsContractAddressRessource = contracts.addResource("{contract_address}");

  // Get all contracts OK
  contracts.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getContractsLambda(scope), { proxy: true })
  );

  // Get a specific contract OK
  contractsContractAddressRessource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getContractLambda(scope), { proxy: true })
  );
  return api;
}
