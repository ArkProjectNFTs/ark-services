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
import { exportToPostman } from "./postman";
import { deployIndexer } from "./ecs/indexer";
import { Vpc } from "aws-cdk-lib/aws-ec2";

// const cacheSettings = {
//   cacheTtl: cdk.Duration.minutes(5),
//   dataEncrypted: true,
// };

export class ArkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: ArkStackProps) {
    super(scope, id, props);

    const vpc = Vpc.fromLookup(this, "ArkVPC", {
      vpcId: "vpc-0d11f7ec183208e08",
    });

    const environement = props.isProductionEnvironment
      ? "production"
      : "staging";
    const apiName = `ark-project-api-${environement}`;

    const api = new apigateway.RestApi(
      this,
      `ark-project-api-gateway-${environement}`,
      {
        restApiName: apiName,
        deploy: false, // Important: Disable automatic deployment
      }
    );

    const basicPlan = api.addUsagePlan(`ark-basic-plan-${environement}`, {
      name: `ark-basic-plan-${environement}`,
      throttle: {
        rateLimit: 5, // 5 requests per second
        burstLimit: 2, // Allow a burst of 2 requests
      },
      quota: {
        limit: 100000, // 100000 requests per month
        period: apigateway.Period.MONTH,
      },
    });

    const payAsYouGoPlan = api.addUsagePlan(
      `ark-pay-as-you-go-plan-${environement}`,
      {
        name: `ark-pay-as-you-go-plan-${environement}`,
        throttle: {
          rateLimit: 100, // 100 requests per second
          burstLimit: 50, // Allow a burst of 50 requests
        },
      }
    );

    const adminPlan = api.addUsagePlan(`ark-admin-plan-${environement}`, {
      name: `ark-admin-plan-${environement}`,
      // No throttle means it's unlimited
    });

    // V1 API
    const versionedRoot = api.root.addResource("v1");

    versionedRoot.addCorsPreflight({
      allowOrigins: apigateway.Cors.ALL_ORIGINS, // or specify the exact origins
      allowMethods: apigateway.Cors.ALL_METHODS, // or specify the exact methods
      // you can also add other headers, allowCredentials, etc.
    });

    const tableNamePrefix = props.isProductionEnvironment
      ? "ark_project"
      : "ark_project_staging";

    contractsApi(this, vpc, versionedRoot, props.stages, tableNamePrefix);
    eventsApi(this, versionedRoot, props.stages, tableNamePrefix);
    tokensApi(this, versionedRoot, props.stages, tableNamePrefix);
    ownerApi(this, versionedRoot, props.stages, tableNamePrefix);

    const postmanApiKey = process.env.POSTMAN_API_KEY || "";
    const awsRegion = process.env.AWS_REGION || "";

    //loop foreach stage in props.stages
    props.stages.forEach(async (stage: string) => {
      const tableName = `${tableNamePrefix}_${stage}`;

      const createdStage = this.createStage(
        api,
        environement,
        stage,
        tableName
      );
      // Add basic plan to API
      basicPlan.addApiStage({ stage: createdStage });
      // Add pay as you go plan to API
      payAsYouGoPlan.addApiStage({ stage: createdStage });
      // Add admin plan to API
      adminPlan.addApiStage({ stage: createdStage });
      if (props.isProductionEnvironment) {
        await exportToPostman(
          environement,
          stage,
          postmanApiKey,
          api.restApiId,
          awsRegion
        );
      }
    });

    deployIndexer(
      this,
      vpc,
      props.isProductionEnvironment,
      props.indexerVersion
    );
  }

  private createStage(
    api: apigateway.RestApi,
    apiSuffix: string,
    stageName: string,
    tableName: string
  ) {
    // Create deployment
    const deployment = new apigateway.Deployment(
      this,
      `ark-project-deployment-${apiSuffix}-${stageName}`,
      { api }
    );

    // Create a log group for the stage
    const stageLogGroup = new logs.LogGroup(
      this,
      `ark-project-log-${stageName}`
    );

    let lambdaUsageTable: string = "default";

    if (apiSuffix === "production") {
      lambdaUsageTable = "ark_lambda_usage";
    } else if (apiSuffix === "staging") {
      lambdaUsageTable = "ark_lambda_usage_staging";
    } else {
      lambdaUsageTable = "ark_lambda_usage_prs";
    }

    // Create stage and point it to the latest deployment
    const stage = new apigateway.Stage(
      this,
      `ark-project-stage-${apiSuffix}-${stageName}`,
      {
        deployment,
        stageName,
        variables: {
          tableName,
          paginationCache: "redis://ipfs.arkproject.dev:6379",
          maxItemsLimit: "100",
          lambdaUsageTable: lambdaUsageTable,
          stageName: stageName,
          sqlxUrl:
            "postgres://postgres:W2sJcY-t@34.65.137.143:5432/ark_project",
        },
        accessLogDestination: new apigateway.LogGroupLogDestination(
          stageLogGroup
        ),
        accessLogFormat: apigateway.AccessLogFormat.jsonWithStandardFields(),
        cachingEnabled: true,
        cacheTtl: cdk.Duration.seconds(0),
        cacheDataEncrypted: true,
        cacheClusterSize: "0.5",
        // activate cache for specific endpoints
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

    return stage;
  }
}
