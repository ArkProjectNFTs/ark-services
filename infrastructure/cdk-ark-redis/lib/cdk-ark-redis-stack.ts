import * as cdk from "aws-cdk-lib";
import { Peer, Port, SecurityGroup, Vpc } from "aws-cdk-lib/aws-ec2";
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

    const environement = props.isProductionEnvironment
      ? "production"
      : "staging";
    const environementName =
      environement.charAt(0).toUpperCase() + environement.slice(1);

    const subnetGroup = new CfnSubnetGroup(
      this,
      `Ark${environementName}RedisSubnetGroup`,
      {
        description: `Subnet Group for Redis Cluster (${environementName})`,
        subnetIds: vpc.privateSubnets.map((subnet) => subnet.subnetId),
      }
    );

    const redisSecurityGroup = new SecurityGroup(
      this,
      `Ark${environementName}RedisSecurityGroup`,
      {
        vpc,
        description: `Security group for Redis Cluster (${environementName})`,
        allowAllOutbound: true,
        securityGroupName: `Ark${environementName}RedisSecurityGroup`,
      }
    );

    const cluster = new CfnCacheCluster(
      this,
      `Ark${environementName}RedisCluster`,
      {
        clusterName: `Ark${environementName}RedisCluster`,
        engine: "redis",
        cacheNodeType: props.isProductionEnvironment
          ? "cache.t3.small"
          : "cache.t2.micro",
        numCacheNodes: 1,
        cacheSubnetGroupName: subnetGroup.ref,
        vpcSecurityGroupIds: [redisSecurityGroup.securityGroupId],
      }
    );

    new ssm.StringParameter(
      this,
      `ArkProject-${environementName}-RedisEndpointAddress`,
      {
        parameterName: `/ark/${environement}/redisConnectionString`,
        stringValue: cluster.attrRedisEndpointAddress,
      }
    );

    new ssm.StringParameter(
      this,
      `ArkProject-${environementName}-RedisSecurityGroupId`,
      {
        parameterName: `/ark/${environement}/redisSecurityGroupId`,
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
