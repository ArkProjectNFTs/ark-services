// infra/lib/infra-stack.ts
import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { contractsApi } from "./apis/contracts-api";
import { eventsApi } from "./apis/events-api";
import { tokensApi } from "./apis/tokens-api";
import { ownerApi } from "./apis/owner-api";

export class ArkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const api = new apigateway.RestApi(this, "ArkProjectApi", {
      restApiName: "ark-project-api",
      deployOptions: {
        stageName: "dev",
      },
    });
    contractsApi(this, api);
    eventsApi(this, api);
    tokensApi(this, api);
    ownerApi(this, api);
  }
}
