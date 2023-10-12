import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { contractsApi } from "./apis/contracts-api";
import { eventsApi } from "./apis/events-api";
import { tokensApi } from "./apis/tokens-api";
import { ownerApi } from "./apis/owner-api";
import { ArkStackProps } from "./types";
import * as acm from "aws-cdk-lib/aws-certificatemanager";
import * as route53 from "aws-cdk-lib/aws-route53";

export class ArkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: ArkStackProps) {
    super(scope, id, props);
    const stageName = props.branch === "main" ? "prod" : "dev";
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

    new route53.CnameRecord(this, "ApiGatewayCnameRecord", {
      recordName: subDomainName,
      zone: hostedZone,
      domainName: customDomain.domainNameAliasDomainName,
    });

    contractsApi(this, api, props);
    eventsApi(this, api, props);
    tokensApi(this, api, props);
    ownerApi(this, api, props);
  }
}
