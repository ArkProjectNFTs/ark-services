import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getContractsLambda } from "../../lambdas/v1/get-contracts-lambda";
import { getContractLambda } from "../../lambdas/v1/get-contract-lambda";
import * as cdk from "aws-cdk-lib";
import { ArkStackProps } from "../../types";

export function contractsApi(
  scope: cdk.Stack,
  versionedRoot: apigateway.IResource,
  props: ArkStackProps
) {
  const contracts = versionedRoot.addResource("contracts");
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
  return versionedRoot;
}
