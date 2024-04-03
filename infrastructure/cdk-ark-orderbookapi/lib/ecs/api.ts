import * as cdk from "aws-cdk-lib";
import * as route53 from "aws-cdk-lib/aws-route53";
import * as route53targets from 'aws-cdk-lib/aws-route53-targets';
import * as acm from "aws-cdk-lib/aws-certificatemanager";
import { LogGroup } from "aws-cdk-lib/aws-logs";
import { Cluster } from "aws-cdk-lib/aws-ecs";
import { ApplicationLoadBalancer } from "aws-cdk-lib/aws-elasticloadbalancingv2";

export async function deployApi(
  scope: cdk.Stack,
  networks: string[],
  isProductionEnvironment: boolean,
  vpc: cdk.aws_ec2.IVpc
) {
  const cluster = new Cluster(scope, "ark-orderbook-api", {
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
  const logGroup = new LogGroup(scope, `/ecs/orderbook-api-${network}`, {
    removalPolicy: cdk.RemovalPolicy.DESTROY,
    retention: 7,
  });

  const taskDefinition = new cdk.aws_ecs.FargateTaskDefinition(
    scope,
    `ark-orderbook-api-${network}-task-definition`,
    {
      memoryLimitMiB: 2048,
      cpu: 512,
    }
  );

  const ecrRepository = cdk.aws_ecr.Repository.fromRepositoryName(
    scope,
    "ArkProjectRepository",
    "ark-project-repo"
  );

  const container = taskDefinition.addContainer("ark_orderbook_api", {
    image: cdk.aws_ecs.ContainerImage.fromEcrRepository(
      ecrRepository,
      isProductionEnvironment
        ? "ark-orderbook-api-production-latest"
        : "ark-orderbook-api-staging-latest"
    ),
    logging: cdk.aws_ecs.LogDrivers.awsLogs({
      streamPrefix: "ecs",
      logGroup: logGroup,
    }),
    environment: {
      RUST_LOG: "DEBUG",
      DATABASE_URL: process.env.DATABASE_URL || "defaultUrl",
      API_USER: process.env.API_USER || "",
      API_PASSWORD: process.env.API_PASSWORD || "",
    },
  });

  const domainName = 'arkproject.dev';
  const subdomainEnvName = isProductionEnvironment ? "" : "staging";
  const apiURL = `${subdomainEnvName}api-orderbook.${domainName}`;

  const hostedZone = route53.HostedZone.fromHostedZoneAttributes(scope, 'HostedZone', {
    hostedZoneId: 'Z057403917YO7G55AYYF9',
    zoneName: domainName,
  });
  const certificate = new acm.Certificate(scope, 'Certificate', {
    domainName: apiURL,
    validation: acm.CertificateValidation.fromDns(hostedZone),
  });

  container.addPortMappings({
    containerPort: 8080,
    protocol: cdk.aws_ecs.Protocol.TCP,
  });


  const fargateService = new cdk.aws_ecs.FargateService(scope, `ark-orderbook-api-${network}`, {
    cluster: cluster,
    taskDefinition: taskDefinition,
    desiredCount: 1,
    securityGroups: [ecsSecurityGroup]
  });

  const lbSecurityGroup = new cdk.aws_ec2.SecurityGroup(scope, 'LBSecurityGroup', {
    vpc,
    description: 'Security group for the Load Balancer',
  });

  lbSecurityGroup.addIngressRule(cdk.aws_ec2.Peer.anyIpv4(), cdk.aws_ec2.Port.tcp(80), 'Allow HTTP traffic from anywhere');
  lbSecurityGroup.addEgressRule(ecsSecurityGroup, cdk.aws_ec2.Port.tcp(8080), 'Allow outbound traffic to ECS security group on port 8080');


  ecsSecurityGroup.addIngressRule(
    lbSecurityGroup,
    cdk.aws_ec2.Port.tcp(8080),
    "Allow inbound HTTP traffic from the load balancer"
  );

  const loadBalancer = new ApplicationLoadBalancer(scope, "ApiLoadBalancer", {
    vpc: vpc,
    internetFacing: true,
    securityGroup: lbSecurityGroup
  });

  const listener = loadBalancer.addListener("Listener", {
    port: 443,
    open: true,
    certificates: [certificate],
});

  listener.addTargets(`FargateServiceTarget-${network}`, {
    port: 80,
    targets: [fargateService.loadBalancerTarget({
      containerName: "ark_orderbook_api",
      containerPort: 8080,
    })],
    healthCheck: {
      path: "/health",
      interval: cdk.Duration.seconds(30),
      timeout: cdk.Duration.seconds(5),
      healthyThresholdCount: 5,
      unhealthyThresholdCount: 2,
    },
  });

  new route53.ARecord(scope, 'ApiOrderbookAliasRecord', {
    zone: hostedZone,
    recordName: apiURL,
    target: route53.RecordTarget.fromAlias(new route53targets.LoadBalancerTarget(loadBalancer)),
  });


}
