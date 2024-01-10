import * as cdk from "aws-cdk-lib";
import { CfnCacheCluster } from "aws-cdk-lib/aws-elasticache";
import { Construct } from "constructs";

export class ArkRedisStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    new CfnCacheCluster(this, "RedisCluster", {
      engine: "redis",
      cacheNodeType: "cache.t3.micro",
      numCacheNodes: 1,
    });
  }
}
