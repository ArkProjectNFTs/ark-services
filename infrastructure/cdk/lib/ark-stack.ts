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

export class ArkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: ArkStackProps) {
    super(scope, id, props);
    const stageName = props.branch === "main" ? "prod" : "staging";
    const apiName = `ark-project-${props.envType}-api`;

    const subdomainStageName = props.branch === "main" ? "" : "staging.";
    const subdomainEnvName = props.envType === "mainnet" ? "" : "-testnet";
    const domainName = "arkproject.dev";
    const subDomainName = `${subdomainStageName}api${subdomainEnvName}.${domainName}`;

    // Fetch the hosted zone and create a CNAME record
    const hostedZone = route53.HostedZone.fromLookup(this, "HostedZone", {
      domainName: domainName,
    });

    // Create an ACM certificate
    const certificate = new acm.Certificate(this, "ApiCertificate", {
      domainName: subDomainName,
      validation: acm.CertificateValidation.fromDns(hostedZone), // Use DNS validation
    });

    // Create the API Gateway without the domain
    const api = new apigateway.RestApi(this, "ArkProjectApi", {
      restApiName: apiName,
      deployOptions: {
        stageName: stageName,
      },
    });

    const deploymentStage = api.deploymentStage;

    // Basic Free Plan
    const basicPlan = api.addUsagePlan("ArkBasicPlan", {
      name: "ArkApiBasic",
      throttle: {
        rateLimit: 5, // 5 requests per second
        burstLimit: 2, // Allow a burst of 2 requests
      },
      quota: {
        limit: 100000, // 100000 requests per month
        period: apigateway.Period.MONTH,
      },
    });

    // Add basic plan to API
    basicPlan.addApiStage({
      stage: deploymentStage,
    });

    // Pay As You Go Plan
    const payAsYouGoPlan = api.addUsagePlan("ArkPayAsYouGoPlan", {
      name: "ArkApiPayAsYouGo",
      throttle: {
        rateLimit: 100, // 100 requests per second
        burstLimit: 50, // Allow a burst of 50 requests
      },
    });

    // Add pay as you go plan to API
    payAsYouGoPlan.addApiStage({
      stage: deploymentStage,
    });

    // Admin Unlimited Plan
    const adminPlan = api.addUsagePlan("ArkAdminPlan", {
      name: "ArkApiAdmin",
      // No throttle means it's unlimited
    });

    // Add admin plan to API
    adminPlan.addApiStage({
      stage: deploymentStage,
    });

    // Create a custom domain name
    const customDomain = new apigateway.DomainName(
      this,
      `ApiCustomDomain${props.branch}${props.envType}`,
      {
        domainName: subDomainName,
        certificate: certificate,
        endpointType: apigateway.EndpointType.EDGE, // or REGIONAL based on your needs
      }
    );

    // Associate the custom domain with the API
    new apigateway.BasePathMapping(this, "BasePathMapping", {
      domainName: customDomain,
      restApi: api,
    });

    // Create a CNAME record for the custom domain
    new route53.CnameRecord(this, "ApiGatewayCnameRecord", {
      recordName: subDomainName,
      zone: hostedZone,
      domainName: customDomain.domainNameAliasDomainName,
    });

    // V1 API
    const versionedRoot = api.root.addResource("v1");
    contractsApi(this, versionedRoot, props);
    eventsApi(this, versionedRoot, props);
    tokensApi(this, versionedRoot, props);
    ownerApi(this, versionedRoot, props);
  }
}
