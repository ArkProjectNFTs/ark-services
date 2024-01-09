import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import { Peer, Port, SecurityGroup, Vpc } from "aws-cdk-lib/aws-ec2";
import { CfnCacheCluster, CfnSubnetGroup } from "aws-cdk-lib/aws-elasticache";

export class ArkRedisStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const vpc = Vpc.fromLookup(this, "ArkVPC", {
      vpcId: "vpc-0d11f7ec183208e08",
    });

    const securityGroup = new SecurityGroup(this, "RedisSecurityGroup", {
      vpc,
      description: "Allow redis access",
      allowAllOutbound: true,
    });

    securityGroup.addIngressRule(
      Peer.anyIpv4(),
      Port.tcp(6379),
      "Allow Redis access"
    );

    new CfnCacheCluster(this, "RedisCluster", {
      engine: "redis",
      cacheNodeType: "cache.t3.micro",
      numCacheNodes: 1,
      vpcSecurityGroupIds: [securityGroup.securityGroupId],
      cacheSubnetGroupName: new CfnSubnetGroup(this, "RedisSubnetGroup", {
        description: "Subnet group for Redis",
        subnetIds: vpc.publicSubnets.map((subnet) => subnet.subnetId),
      }).cacheSubnetGroupName,
    });
  }
}
