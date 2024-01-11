import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getContractsLambda } from "../../lambdas/v1/get-contracts-lambda";
import { getContractLambda } from "../../lambdas/v1/get-contract-lambda";
import * as cdk from "aws-cdk-lib";
import { IVpc, SecurityGroup } from "aws-cdk-lib/aws-ec2";

export function contractsApi(
  scope: cdk.Stack,
  vpc: IVpc,
  lambdaSecurityGroup: SecurityGroup,
  versionedRoot: apigateway.IResource,
  stages: string[],
  tableNamePrefix: string
) {
  const contracts = versionedRoot.addResource("contracts");
  const contractsContractAddressRessource =
    contracts.addResource("{contract_address}");

  // Get all contracts
  contracts.addMethod(
    "GET",
    new apigateway.LambdaIntegration(
      getContractsLambda(
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

  // Get a specific contract
  contractsContractAddressRessource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(
      getContractLambda(scope, stages, tableNamePrefix),
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
