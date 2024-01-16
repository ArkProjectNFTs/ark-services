import * as cdk from "aws-cdk-lib";
import { IVpc, Peer, Port, SecurityGroup, Vpc } from "aws-cdk-lib/aws-ec2";
import { Construct } from "constructs";
import { ArkApiStackProps } from "./types";
import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { contractsApi } from "./apis/v1/contracts-api";
import { eventsApi } from "./apis/v1/events-api";
import { tokensApi } from "./apis/v1/tokens-api";
import { ownerApi } from "./apis/v1/owner-api";
import createApiStage from "./apis/create-stage";
// import { exportToPostman } from "./postman";
import { deployBlockIndexerLambda } from "./lambdas/block-indexer";
import { setupApiPlans } from "./apis/create-plans";
import * as ssm from "aws-cdk-lib/aws-ssm";

export class ArkApiStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: ArkApiStackProps) {
    super(scope, id, props);

    const awsRegion = process.env.AWS_REGION || "";

    const environment = this.getEnvironment(props);

    const environmentName =
      environment.charAt(0).toUpperCase() + environment.slice(1);

    const vpc = this.getVpc();
    const api = this.createApi(environment);

    const { adminPlan, basicPlan, payAsYouGoPlan } = setupApiPlans(
      api,
      environment
    );

    // V1 API
    const versionedRoot = api.root.addResource("v1");

    versionedRoot.addCorsPreflight({
      allowOrigins: apigateway.Cors.ALL_ORIGINS,
      allowMethods: apigateway.Cors.ALL_METHODS,
    });

    const tableNamePrefix = props.isProductionEnvironment
      ? "ark_project"
      : "ark_project_staging";

    const lambdaSecurityGroup = new SecurityGroup(
      this,
      "ark-lambda-security-group",
      {
        securityGroupName: "ark-lambda-security-group",
        vpc,
        description: "Security group for Lambdas",
        allowAllOutbound: true,
      }
    );

    const securityGroupId = ssm.StringParameter.valueForStringParameter(
      this,
      `/ark/${environment}/redisSecurityGroupId`
    );

    const redisSecurityGroup = SecurityGroup.fromSecurityGroupId(
      this,
      `ark-redis-security-group-${environment}`,
      securityGroupId
    );

    redisSecurityGroup.addIngressRule(
      Peer.securityGroupId(lambdaSecurityGroup.securityGroupId),
      Port.tcp(6379),
      "Allow inbound Redis traffic from Lambda security group"
    );

    contractsApi(
      this,
      vpc,
      lambdaSecurityGroup,
      versionedRoot,
      props.stages,
      tableNamePrefix
    );
    eventsApi(
      this,
      vpc,
      lambdaSecurityGroup,
      versionedRoot,
      props.stages,
      tableNamePrefix
    );
    tokensApi(
      this,
      vpc,
      lambdaSecurityGroup,
      versionedRoot,
      props.stages,
      tableNamePrefix
    );
    ownerApi(
      this,
      vpc,
      lambdaSecurityGroup,
      versionedRoot,
      props.stages,
      tableNamePrefix
    );

    ["mainnet", "testnet"].forEach((network) => {
      deployBlockIndexerLambda(
        this,
        vpc,
        lambdaSecurityGroup,
        `block-indexer-${network}-${
          props.isProductionEnvironment ? "production" : "staging"
        }`,
        network,
        `${tableNamePrefix}_${network}`,
        environment
      );
    });

    //loop foreach stage in props.stages
    props.stages.forEach(async (stage: string) => {
      const createdStage = createApiStage(
        this,
        api,
        environment,
        stage,
        `${tableNamePrefix}_${stage}`
      );
      // Add basic plan to API
      basicPlan.addApiStage({ stage: createdStage });
      // Add pay as you go plan to API
      payAsYouGoPlan.addApiStage({ stage: createdStage });
      // Add admin plan to API
      adminPlan.addApiStage({ stage: createdStage });

      // if (props.isProductionEnvironment) {
      //   const postmanApiKey = process.env.POSTMAN_API_KEY || "";
      //   await exportToPostman(
      //     environment,
      //     stage,
      //     postmanApiKey,
      //     api.restApiId,
      //     awsRegion
      //   );
      // }
    });
  }

  private getVpc(): IVpc {
    const vpcId = "vpc-0d11f7ec183208e08";
    return Vpc.fromLookup(this, "ArkVPC", { vpcId });
  }

  private getEnvironment(props: ArkApiStackProps): string {
    return props.isProductionEnvironment ? "production" : "staging";
  }

  private createApi(environment: string): apigateway.RestApi {
    return new apigateway.RestApi(
      this,
      `ark-project-api-gateway-${environment}`,
      {
        restApiName: `ark-project-api-${environment}`,
        deploy: false, // Disable automatic deployment
      }
    );
  }
}
