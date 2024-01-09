import * as cdk from "aws-cdk-lib";
import { Peer, Port, SecurityGroup, Vpc } from "aws-cdk-lib/aws-ec2";
import { CfnCacheCluster, CfnSubnetGroup } from "aws-cdk-lib/aws-elasticache";
import { Construct } from "constructs";

export class ArkRedisStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // Utilisation de Vpc.fromLookup pour obtenir un VPC existant par son ID
    const vpc = Vpc.fromLookup(this, "ArkVPC", {
      vpcId: "vpc-0d11f7ec183208e08",
    });

    // Création d'un Security Group pour Redis dans ce VPC
    const securityGroup = new SecurityGroup(this, "RedisSecurityGroup", {
      vpc,
      description: "Allow redis access",
      allowAllOutbound: true,
    });

    // Autoriser les connexions entrantes sur le port 6379 pour Redis
    securityGroup.addIngressRule(
      Peer.anyIpv4(),
      Port.tcp(6379),
      "Allow Redis access"
    );

    // Création du Subnet Group pour Redis
    const redisSubnetGroup = new CfnSubnetGroup(this, "RedisSubnetGroup", {
      description: "Subnet group for Redis",
      subnetIds: vpc.privateSubnets.map((subnet) => subnet.subnetId),
    });

    // Création du cluster Redis
    new CfnCacheCluster(this, "RedisCluster", {
      engine: "redis",
      cacheNodeType: "cache.t3.micro",
      numCacheNodes: 1,
      vpcSecurityGroupIds: [securityGroup.securityGroupId],
      cacheSubnetGroupName: redisSubnetGroup.cacheSubnetGroupName,
    });
  }
}
