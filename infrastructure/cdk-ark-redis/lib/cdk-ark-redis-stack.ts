import * as cdk from "aws-cdk-lib";
import { SecurityGroup, Vpc } from "aws-cdk-lib/aws-ec2";
import { CfnCacheCluster, CfnSubnetGroup } from "aws-cdk-lib/aws-elasticache";
import { Construct } from "constructs";
import * as ssm from "aws-cdk-lib/aws-ssm";

interface ArkRedisStackProps extends cdk.StackProps {
  isProductionEnvironment: boolean;
}

export class ArkRedisStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: ArkRedisStackProps) {
    super(scope, id, props);

    const vpc = Vpc.fromLookup(this, "ArkVPC", {
      vpcId: "vpc-0d11f7ec183208e08",
    });

    const environment = props.isProductionEnvironment
      ? "production"
      : "staging";
    const environmentName =
      environment.charAt(0).toUpperCase() + environment.slice(1);

    const subnetGroup = new CfnSubnetGroup(
      this,
      `ark-redis-subnet-group-${environment}`,
      {
        cacheSubnetGroupName: `ark-redis-subnet-group-${environment}`,
        description: `Subnet Group for Redis Cluster (${environment})`,
        subnetIds: vpc.privateSubnets.map((subnet) => subnet.subnetId),
      }
    );

    const redisSecurityGroup = new SecurityGroup(
      this,
      `ark-redis-security-group-${environment}`,
      {
        vpc,
        description: `Security group for Redis Cluster (${environmentName})`,
        allowAllOutbound: true,
        securityGroupName: `ark-redis-security-group-${environment}`,
      }
    );

    const cluster = new CfnCacheCluster(
      this,
      `ark-redis-cluster-${environment}`,
      {
        clusterName: `ark-redis-cluster-${environment}`,
        engine: "redis",
        cacheNodeType: props.isProductionEnvironment
          ? "cache.t3.small"
          : "cache.t2.micro",
        numCacheNodes: 1,
        cacheSubnetGroupName: subnetGroup.ref,
        vpcSecurityGroupIds: [redisSecurityGroup.securityGroupId],
      }
    );

    new ssm.StringParameter(this, `ark-redis-endpoint-address-${environment}`, {
      parameterName: `/ark/${environment}/redisConnectionString`,
      stringValue: cluster.attrRedisEndpointAddress,
    });

    new ssm.StringParameter(
      this,
      `ark-redis-security-group-id-${environment}`,
      {
        parameterName: `/ark/${environment}/redisSecurityGroupId`,
        stringValue: redisSecurityGroup.securityGroupId,
      }
    );

    new cdk.CfnOutput(this, `redisConnectionString`, {
      value: cluster.attrRedisEndpointAddress,
    });

    new cdk.CfnOutput(this, "redisSecurityGroupId", {
      value: redisSecurityGroup.securityGroupId,
    });
  }
}
