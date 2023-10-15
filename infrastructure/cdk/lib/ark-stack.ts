import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { contractsApi } from "./apis/v1/contracts-api";
import { eventsApi } from "./apis/v1/events-api";
import { tokensApi } from "./apis/v1/tokens-api";
import { ownerApi } from "./apis/v1/owner-api";
import { ArkStackProps } from "./types";
import * as acm from "aws-cdk-lib/aws-certificatemanager";
import * as route53 from "aws-cdk-lib/aws-route53";
import * as logs from "aws-cdk-lib/aws-logs";

const cacheSettings = {
  cacheTtl: cdk.Duration.minutes(5),
  dataEncrypted: true,
};

export class ArkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: ArkStackProps) {
    super(scope, id, props);
    let apiSuffix: string;

    if (props.isPullRequest) {
      apiSuffix = props.branch;
    } else if (props.branch === "main") {
      apiSuffix = "production";
    } else {
      apiSuffix = "staging";
    }

    const apiName = `ark-project-api-${apiSuffix}${
      props.isPullRequest ? "_pr" : ""
    }`;

    const api = new apigateway.RestApi(
      this,
      `ark-project-api-gateway-${apiSuffix}${props.isPullRequest ? "_pr" : ""}`,
      {
        restApiName: apiName,
        deploy: false, // Important: Disable automatic deployment
      }
    );

    // V1 API
    const versionedRoot = api.root.addResource("v1");

    versionedRoot.addCorsPreflight({
      allowOrigins: apigateway.Cors.ALL_ORIGINS, // or specify the exact origins
      allowMethods: apigateway.Cors.ALL_METHODS, // or specify the exact methods
      // you can also add other headers, allowCredentials, etc.
    });

    contractsApi(this, versionedRoot, props.stages);
    eventsApi(this, versionedRoot, props.stages);
    tokensApi(this, versionedRoot, props.stages);
    ownerApi(this, versionedRoot, props.stages);

    //loop foreach stage in props.stages
    props.stages.forEach((stage: string) => {
      this.createStage(api, apiSuffix, stage, props.isPullRequest);
    });
  }

  private createStage(
    api: apigateway.RestApi,
    apiSuffix: string,
    stageName: string,
    isPullRequest: boolean
  ) {
    // Create deployment
    const deployment = new apigateway.Deployment(
      this,
      `ark-project-deployment-${apiSuffix}-${stageName}${
        isPullRequest ? "_pr" : ""
      }`,
      { api }
    );

    // Create a log group for the stage
    const stageLogGroup = new logs.LogGroup(
      this,
      `ark-project-log-${stageName}${isPullRequest ? "_pr" : ""}`
    );

    // Create stage and point it to the latest deployment
    const stage = new apigateway.Stage(
      this,
      `ark-project-stage-${apiSuffix}-${stageName}${
        isPullRequest ? "_pr" : ""
      }`,
      {
        deployment,
        stageName,
        variables: {
          tableName: `ark_project_${stageName}`,
          paginationCache: "redis://ipfs.arkproject.dev:6379",
          maxItemsLimit: "100",
        },
        accessLogDestination: new apigateway.LogGroupLogDestination(
          stageLogGroup
        ),
        accessLogFormat: apigateway.AccessLogFormat.jsonWithStandardFields(),
        cachingEnabled: true,
        cacheTtl: cdk.Duration.seconds(0),
        cacheDataEncrypted: true,
        cacheClusterSize: "0.5",
        // methodOptions: {
        //   "/contracts/*": {
        //     cachingEnabled: true,
        //     cacheDataEncrypted: cacheSettings.dataEncrypted,
        //     cacheTtl: cacheSettings.cacheTtl,
        //   },
        //   "/events/*": {
        //     cachingEnabled: true,
        //     cacheDataEncrypted: cacheSettings.dataEncrypted,
        //     cacheTtl: cacheSettings.cacheTtl,
        //   },
        //   "/tokens/*": {
        //     cachingEnabled: true,
        //     cacheDataEncrypted: cacheSettings.dataEncrypted,
        //     cacheTtl: cacheSettings.cacheTtl,
        //   },
        // },
      }
    );

    // // Basic Free Plan
    // const basicPlan = api.addUsagePlan("ArkBasicPlan", {
    //   name: "ArkApiBasic",
    //   throttle: {
    //     rateLimit: 5, // 5 requests per second
    //     burstLimit: 2, // Allow a burst of 2 requests
    //   },
    //   quota: {
    //     limit: 100000, // 100000 requests per month
    //     period: apigateway.Period.MONTH,
    //   },
    // });

    // // Add basic plan to API
    // basicPlan.addApiStage({
    //   stage: stage,
    // });

    // // Pay As You Go Plan
    // const payAsYouGoPlan = api.addUsagePlan("ArkPayAsYouGoPlan", {
    //   name: "ArkApiPayAsYouGo",
    //   throttle: {
    //     rateLimit: 100, // 100 requests per second
    //     burstLimit: 50, // Allow a burst of 50 requests
    //   },
    // });

    // // Add pay as you go plan to API
    // payAsYouGoPlan.addApiStage({
    //   stage: stage,
    // });

    // // Admin Unlimited Plan
    // const adminPlan = api.addUsagePlan("ArkAdminPlan", {
    //   name: "ArkApiAdmin",
    //   // No throttle means it's unlimited
    // });

    // // Add admin plan to API
    // adminPlan.addApiStage({
    //   stage: stage,
    // });

    // If this is a pull request, don't create a custom domain
    if (!isPullRequest) {
      const domainName = "arkproject.dev";
      const subdomainEnvName = apiSuffix === "production" ? "" : "staging.";
      const subdomainStageName = stageName === "mainnet" ? "" : "testnet-";
      const apiURL = `${subdomainEnvName}${subdomainStageName}api.${domainName}`;

      // Fetch the hosted zone and create a CNAME record
      const hostedZone = route53.HostedZone.fromLookup(
        this,
        `ark-project-hosted-zone-${apiSuffix}-${stageName}`,
        {
          domainName: domainName,
        }
      );

      // Create an ACM certificate
      const certificate = new acm.Certificate(
        this,
        `ark-project-certificate-${apiSuffix}-${stageName}`,
        {
          domainName: apiURL,
          validation: acm.CertificateValidation.fromDns(hostedZone), // Use DNS validation
        }
      );

      // Create a custom domain name
      const customDomain = new apigateway.DomainName(
        this,
        `ark-project-custom-domain-${apiSuffix}-${stageName}`,
        {
          domainName: apiURL,
          certificate: certificate,
          endpointType: apigateway.EndpointType.REGIONAL,
        }
      );

      // Associate the custom domain with the stage
      new apigateway.BasePathMapping(
        this,
        `ark-project-basepath-mapping-${apiSuffix}-${stageName}`,
        {
          domainName: customDomain,
          restApi: api,
          stage: stage,
        }
      );

      // Create a CNAME record for the custom domain
      new route53.CnameRecord(
        this,
        `ark-project-cname-record-${apiSuffix}-${stageName}`,
        {
          recordName: apiURL,
          zone: hostedZone,
          domainName: customDomain.domainNameAliasDomainName,
        }
      );
    }
  }
}
