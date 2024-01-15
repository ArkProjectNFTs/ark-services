import * as apigateway from "aws-cdk-lib/aws-apigateway";
import * as cdk from "aws-cdk-lib";
import * as logs from "aws-cdk-lib/aws-logs";
import * as route53 from "aws-cdk-lib/aws-route53";
import * as acm from "aws-cdk-lib/aws-certificatemanager";
import * as ssm from "aws-cdk-lib/aws-ssm";

function createApiStage(
  scope: cdk.Stack,
  api: apigateway.RestApi,
  environement: string,
  stageName: string,
  tableName: string
) {
  // Create deployment
  const deployment = new apigateway.Deployment(
    scope,
    `ark-project-deployment-${environement}-${stageName}`,
    { api }
  );

  // Create a log group for the stage
  const stageLogGroup = new logs.LogGroup(
    scope,
    `ark-project-log-${stageName}`
  );

  let lambdaUsageTable: string = "default";

  if (environement === "production") {
    lambdaUsageTable = "ark_lambda_usage";
  } else if (environement === "staging") {
    lambdaUsageTable = "ark_lambda_usage_staging";
  } else {
    lambdaUsageTable = "ark_lambda_usage_prs";
  }

  const redisConnectionString = ssm.StringParameter.valueForStringParameter(
    scope,
    `/ark/${environement}/redisConnectionString`
  );

  console.log("=> redisConnectionString", redisConnectionString);

  // Create stage and point it to the latest deployment
  const stage = new apigateway.Stage(
    scope,
    `ark-project-stage-${environement}-${stageName}`,
    {
      deployment,
      stageName,
      variables: {
        tableName,
        paginationCache: `redis://${redisConnectionString}:6379`,
        maxItemsLimit: "100",
        lambdaUsageTable: lambdaUsageTable,
        stageName: stageName,
        sqlxUrl: "postgres://postgres:Pnv4nk2mf@35.237.127.105:5432/postgres",
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
  const subdomainEnvName = environement === "production" ? "" : "staging.";
  const subdomainStageName = stageName === "mainnet" ? "" : "testnet-";
  const apiURL = `${subdomainEnvName}${subdomainStageName}api.${domainName}`;

  // Fetch the hosted zone and create a CNAME record
  const hostedZone = route53.HostedZone.fromLookup(
    scope,
    `ark-project-hosted-zone-${environement}-${stageName}`,
    {
      domainName: domainName,
    }
  );

  // Create an ACM certificate
  const certificate = new acm.Certificate(
    scope,
    `ark-project-certificate-${environement}-${stageName}`,
    {
      domainName: apiURL,
      validation: acm.CertificateValidation.fromDns(hostedZone), // Use DNS validation
    }
  );

  // Create a custom domain name
  const customDomain = new apigateway.DomainName(
    scope,
    `ark-project-custom-domain-${environement}-${stageName}`,
    {
      domainName: apiURL,
      certificate: certificate,
      endpointType: apigateway.EndpointType.REGIONAL,
    }
  );

  // Associate the custom domain with the stage
  new apigateway.BasePathMapping(
    scope,
    `ark-project-basepath-mapping-${environement}-${stageName}`,
    {
      domainName: customDomain,
      restApi: api,
      stage: stage,
    }
  );

  // Create a CNAME record for the custom domain
  new route53.CnameRecord(
    scope,
    `ark-project-cname-record-${environement}-${stageName}`,
    {
      recordName: apiURL,
      zone: hostedZone,
      domainName: customDomain.domainNameAliasDomainName,
    }
  );

  return stage;
}

export default createApiStage;
