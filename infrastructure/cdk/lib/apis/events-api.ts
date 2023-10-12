import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { getContractEventsLambda } from "../lambdas/get-contract-events-lambda";
import { getTokenEventsLambda } from "../lambdas/get-token-events-lambda";
import * as cdk from "aws-cdk-lib";
import { ArkStackProps } from "../types";

export function eventsApi(
  scope: cdk.Stack,
  api: apigateway.RestApi,
  props: ArkStackProps
) {
  const eventsResource = api.root.addResource("events");
  const eventContractAddressResource =
    eventsResource.addResource("{contract_address}");
  const eventTokenIdResource =
    eventContractAddressResource.addResource("{token_id}");

  // Get all events for a contract
  eventContractAddressResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getContractEventsLambda(scope, props), {
      proxy: true,
    })
  );

  // Get all events for a token
  eventTokenIdResource.addMethod(
    "GET",
    new apigateway.LambdaIntegration(getTokenEventsLambda(scope, props), {
      proxy: true,
    })
  );

  return api;
}
