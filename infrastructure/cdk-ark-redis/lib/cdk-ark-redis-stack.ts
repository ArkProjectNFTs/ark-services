import * as cdk from "aws-cdk-lib";
import { Peer, Port, SecurityGroup, Vpc } from "aws-cdk-lib/aws-ec2";
import { CfnCacheCluster, CfnSubnetGroup } from "aws-cdk-lib/aws-elasticache";
import { Construct } from "constructs";

export class ArkRedisStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const vpc = Vpc.fromLookup(this, "ArkVPC", {
      vpcId: "vpc-0d11f7ec183208e08",
    });

    const subnetGroup = new CfnSubnetGroup(this, "ArkRedisSubnetGroup", {
      description: "Subnet Group for Redis Cluster",
      subnetIds: vpc.privateSubnets.map((subnet) => subnet.subnetId),
    });

    const redisSecurityGroup = new SecurityGroup(this, "RedisSecurityGroup", {
      vpc,
      description: "Security group for Redis Cluster",
      allowAllOutbound: true,
    });

    // redisSecurityGroup.addIngressRule(
    //   Peer.securityGroupId("sgr-0b290e175cbebce5d"), // reference to the Lambda security group ID
    //   Port.tcp(6379),
    //   "Allow inbound Redis traffic from Lambda security group"
    // );

    new CfnCacheCluster(this, "ProdRedisCluster", {
      clusterName: "ProdRedisCluster",
      engine: "redis",
      cacheNodeType: "cache.t2.small",
      numCacheNodes: 1,
      cacheSubnetGroupName: subnetGroup.ref,
      vpcSecurityGroupIds: [redisSecurityGroup.securityGroupId],
    });
  }
}
