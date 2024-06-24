import * as cdk from "aws-cdk-lib";
import * as route53 from "aws-cdk-lib/aws-route53";
import * as route53targets from "aws-cdk-lib/aws-route53-targets";
import * as acm from "aws-cdk-lib/aws-certificatemanager";
import { LogGroup } from "aws-cdk-lib/aws-logs";
import { Cluster } from "aws-cdk-lib/aws-ecs";
import { ApplicationLoadBalancer } from "aws-cdk-lib/aws-elasticloadbalancingv2";
import * as iam from "aws-cdk-lib/aws-iam";

export async function deployApi(
  scope: cdk.Stack,
  networks: string[],
  isProductionEnvironment: boolean,
  vpc: cdk.aws_ec2.IVpc
) {
  const cluster = new Cluster(scope, "marketplace-api", {
    vpc: vpc,
  });

  const ecsSecurityGroup = new cdk.aws_ec2.SecurityGroup(
    scope,
    "ECSSecurityGroup",
    {
      vpc,
      description: "Security group for ECS tasks",
    }
  );

  networks.forEach((network) => {
    deployApiServices(
      isProductionEnvironment,
      scope,
      network,
      cluster,
      ecsSecurityGroup,
      vpc
    );
  });
}

function deployApiServices(
  isProductionEnvironment: boolean,
  scope: cdk.Stack,
  network: string,
  cluster: cdk.aws_ecs.ICluster,
  ecsSecurityGroup: cdk.aws_ec2.SecurityGroup,
  vpc: cdk.aws_ec2.IVpc
) {
  const logGroup = new LogGroup(
    scope,
    `/aws/ecs/marketplace-api-${network}-${
      isProductionEnvironment ? "production" : "staging"
    }`,
    {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      retention: cdk.aws_logs.RetentionDays.ONE_WEEK,
      logGroupName: `/aws/ecs/marketplace-api-${network}-${
        isProductionEnvironment ? "production" : "staging"
      }`,
    }
  );

  const taskDefinition = new cdk.aws_ecs.FargateTaskDefinition(
    scope,
    `marketplace-api-${network}-task-definition`,
    {
      memoryLimitMiB: 2048,
      cpu: 512,
    }
  );

  if (isProductionEnvironment) {
    taskDefinition.taskRole.addToPrincipalPolicy(
      new iam.PolicyStatement({
        actions: ["secretsmanager:GetSecretValue"],
        resources: [
          "arn:aws:secretsmanager:us-east-1:223605539824:secret:prod/ark-db-credentials-AlF3F1",
        ],
      })
    );
  }

  const ecrRepository = cdk.aws_ecr.Repository.fromRepositoryName(
    scope,
    "ArkProjectRepository",
    "ark-project-repo"
  );

  const container = taskDefinition.addContainer("marketplace_api", {
    image: cdk.aws_ecs.ContainerImage.fromEcrRepository(
      ecrRepository,
      `marketplace-api-${
        isProductionEnvironment ? "production" : "staging"
      }-latest`
    ),
    logging: cdk.aws_ecs.LogDrivers.awsLogs({
      streamPrefix: "ecs",
      logGroup: logGroup,
    }),
    environment: {
      RUST_LOG: "INFO",
      AWS_SECRET_NAME: isProductionEnvironment
        ? "prod/ark-db-credentials"
        : "staging/ark-db-credentials",
      REDIS_URL:
        process.env.REDIS_URL || "defaultUrl",
      REDIS_USERNAME:
        process.env.REDIS_URL || "defaultRedisUsername",
      REDIS_PASSWORD:
        process.env.REDIS_PASSWORD || "defaultRedisPassword",
      // DATABASE_URL: process.env.DATABASE_URL || "defaultUrl",
      // API_USER: process.env.API_USER || "",
      // API_PASSWORD: process.env.API_PASSWORD || "",
    },
  });

  const domainName = "arkproject.dev";
  const apiURL = `api.marketplace.${domainName}`;

  const hostedZone = route53.HostedZone.fromHostedZoneAttributes(
    scope,
    "HostedZone",
    {
      hostedZoneId: "Z057403917YO7G55AYYF9",
      zoneName: domainName,
    }
  );

  const certificate = new acm.Certificate(scope, "MarketplaceApiCertificate", {
    domainName: apiURL,
    certificateName: "marketplace-api-certificate",
    validation: acm.CertificateValidation.fromDns(hostedZone),
  });

  container.addPortMappings({
    containerPort: 8080,
    protocol: cdk.aws_ecs.Protocol.TCP,
  });

  const fargateService = new cdk.aws_ecs.FargateService(
    scope,
    `marketplace-api-${network}`,
    {
      cluster: cluster,
      taskDefinition: taskDefinition,
      desiredCount: 1,
      securityGroups: [ecsSecurityGroup],
    }
  );

  const lbSecurityGroup = new cdk.aws_ec2.SecurityGroup(
    scope,
    "LBSecurityGroup",
    {
      vpc,
      description: "Security group for the Load Balancer",
      allowAllOutbound: true,
    }
  );

  lbSecurityGroup.addIngressRule(
    cdk.aws_ec2.Peer.anyIpv4(),
    cdk.aws_ec2.Port.tcp(80),
    "Allow HTTP traffic from anywhere"
  );
  ecsSecurityGroup.addIngressRule(
    lbSecurityGroup,
    cdk.aws_ec2.Port.tcp(8080),
    "Allow inbound HTTP traffic from the load balancer"
  );

  const loadBalancer = new ApplicationLoadBalancer(scope, "ApiLoadBalancer", {
    vpc: vpc,
    internetFacing: true,
    securityGroup: lbSecurityGroup,
  });

  const listener = loadBalancer.addListener("Listener", {
    port: 443,
    open: true,
    certificates: [certificate],
  });

  listener.addTargets(`FargateServiceTarget-${network}`, {
    port: 80,
    targets: [
      fargateService.loadBalancerTarget({
        containerName: "marketplace_api",
        containerPort: 8080,
      }),
    ],
    healthCheck: {
      path: "/health",
      interval: cdk.Duration.seconds(30),
      timeout: cdk.Duration.seconds(5),
      healthyThresholdCount: 5,
      unhealthyThresholdCount: 2,
    },
  });

  new route53.ARecord(scope, "MarketplaceApiAliasRecord", {
    zone: hostedZone,
    recordName: apiURL,
    target: route53.RecordTarget.fromAlias(
      new route53targets.LoadBalancerTarget(loadBalancer)
    ),
  });
}
