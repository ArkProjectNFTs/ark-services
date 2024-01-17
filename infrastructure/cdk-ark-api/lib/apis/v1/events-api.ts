import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getContractEventsLambda } from "../../lambdas/v1/get-contract-events-lambda";
import { getTokenEventsLambda } from "../../lambdas/v1/get-token-events-lambda";
import * as cdk from "aws-cdk-lib";
import { IVpc, SecurityGroup } from "aws-cdk-lib/aws-ec2";
import { getEventsLambda } from "../../lambdas/v1/get-events";

export function eventsApi(
  scope: cdk.Stack,
  vpc: IVpc,
  lambdaSecurityGroup: SecurityGroup,
  versionedRoot: apigateway.IResource,
  stages: string[],
  tableNamePrefix: string
) {
  const eventsResource = versionedRoot.addResource("events");
  const eventContractAddressResource =
    eventsResource.addResource("{contract_address}");
  const eventTokenIdResource =
    eventContractAddressResource.addResource("{token_id}");

  // Get latest events
  eventsResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(
      getEventsLambda(scope, vpc, lambdaSecurityGroup, stages, tableNamePrefix),
      {
        proxy: true,
      }
    ),
    {
      apiKeyRequired: true, // API key is now required for this method
    }
  );

  // Get all events for a contract
  eventContractAddressResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(
      getContractEventsLambda(
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

  // Get all events for a token
  eventTokenIdResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(
      getTokenEventsLambda(
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
