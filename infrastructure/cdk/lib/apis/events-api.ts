import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getContractEventsLambda } from "../lambdas/get-contract-events-lambda";
import { getTokenEventsLambda } from "../lambdas/get-token-events-lambda";
import * as cdk from "aws-cdk-lib";

export function eventsApi(
  scope: cdk.Stack,
  api: apigateway.RestApi
) {
  const eventsResource = api.root.addResource("events");
  const eventContractAddressResource =
    eventsResource.addResource("{contract_address}");
  const eventTokenIdResource =
    eventContractAddressResource.addResource("{token_id}");

  // Get all events for a contract
  eventContractAddressResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getContractEventsLambda(scope), {
      proxy: true,
    })
  );

  // Get all events for a token
  eventTokenIdResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getTokenEventsLambda(scope), {
      proxy: true,
    })
  );

  return api;
}
