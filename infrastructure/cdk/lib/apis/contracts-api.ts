import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getContractsLambda } from "../lambdas/get-contracts-lambda";
import { getContractLambda } from "../lambdas/get-contract-lambda";
import * as cdk from "aws-cdk-lib";
import { ArkStackProps } from "../types";

export function contractsApi(
  scope: cdk.Stack,
  api: apigateway.RestApi,
  props: ArkStackProps
) {
  const contracts = api.root.addResource("contracts");
  const contractsContractAddressRessource =
    contracts.addResource("{contract_address}");

  // Get all contracts
  contracts.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getContractsLambda(scope, props), { proxy: true })
  );

  // Get a specific contract
  contractsContractAddressRessource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getContractLambda(scope, props), { proxy: true })
  );
  return api;
}
