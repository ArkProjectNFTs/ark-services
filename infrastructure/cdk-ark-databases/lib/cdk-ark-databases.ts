import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import * as rds from "aws-cdk-lib/aws-rds";
import * as ec2 from "aws-cdk-lib/aws-ec2";
import * as secretsmanager from "aws-cdk-lib/aws-secretsmanager";
import * as backup from "aws-cdk-lib/aws-backup";
import * as sns from "aws-cdk-lib/aws-sns";
import * as cloudwatch from "aws-cdk-lib/aws-cloudwatch";
import { SnsAction } from "aws-cdk-lib/aws-cloudwatch-actions";

export class ArkDatabasesDeploymentStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // Constants
    const snsTopicArn = "arn:aws:sns:us-east-1:223605539824:ArkProjectErrors";

    // VPC
    const vpc = ec2.Vpc.fromLookup(this, "ArkVPC", {
      vpcId: "vpc-0d11f7ec183208e08",
    });

    // Security Group
    const dbSecurityGroup = new ec2.SecurityGroup(this, "DBSecurityGroup", {
      vpc,
      allowAllOutbound: true,
    });

    dbSecurityGroup.addIngressRule(
      ec2.Peer.anyIpv4(),
      ec2.Port.tcp(5432),
      "Allow PostgreSQL access from external IP range"
    );

    // Secret for RDS credentials
    const dbCredentialsSecret = new secretsmanager.Secret(
      this,
      "DBCredentialsSecret",
      {
        secretName: "prod/ark-db-credentials",
        generateSecretString: {
          secretStringTemplate: JSON.stringify({ username: "dbadmin" }),
          generateStringKey: "password",
          excludePunctuation: true,
        },
      }
    );

    // RDS instance
    const dbInstance = new rds.DatabaseInstance(this, "ArkProjectPostgres", {
      engine: rds.DatabaseInstanceEngine.postgres({
        version: rds.PostgresEngineVersion.VER_15_4,
      }),
      instanceType: ec2.InstanceType.of(
        ec2.InstanceClass.T3,
        ec2.InstanceSize.MICRO
      ),
      parameters: {
        "rds.force_ssl": "0",
      },
      vpc,
      vpcSubnets: {
        subnets: [
          ec2.Subnet.fromSubnetId(
            this,
            "PublicSubnet2",
            "subnet-01fb3b6077af7c052"
          ),
          ec2.Subnet.fromSubnetId(
            this,
            "PublicSubnet1",
            "subnet-043e080e03b9f4ba1"
          ),
        ],
      },
      securityGroups: [dbSecurityGroup],
      credentials: rds.Credentials.fromSecret(dbCredentialsSecret),
      multiAz: false,
      allocatedStorage: 20,
      maxAllocatedStorage: 100,
      storageType: rds.StorageType.GP2,
      backupRetention: cdk.Duration.days(7),
      deletionProtection: true,
      publiclyAccessible: true,
      databaseName: "arkproject",
      enablePerformanceInsights: true,
      performanceInsightRetention: rds.PerformanceInsightRetention.DEFAULT,
    });

    // CloudWatch alarm for CPU utilization
    const cpuAlarm = new cloudwatch.Alarm(this, "ArkProjectDatabaseCPUAlarm", {
      alarmName: "High CPU Utilization Alarm",
      metric: dbInstance.metricCPUUtilization(),
      threshold: 80,
      evaluationPeriods: 1,
      comparisonOperator:
        cloudwatch.ComparisonOperator.GREATER_THAN_OR_EQUAL_TO_THRESHOLD,
      actionsEnabled: true,
      alarmDescription: "This will alarm if CPU utilization is above 80%",
    });

    cpuAlarm.addAlarmAction(
      new SnsAction(
        sns.Topic.fromTopicArn(this, "ArkProjectErrors", snsTopicArn)
      )
    );

    // AWS Backup plan
    const backupPlan = new backup.BackupPlan(this, "BackupPlan", {
      backupPlanName: "ArkDBBackupPlan",
    });

    backupPlan.addSelection("Selection", {
      resources: [backup.BackupResource.fromRdsDatabaseInstance(dbInstance)],
    });

    backupPlan.addRule(
      new backup.BackupPlanRule({
        ruleName: "DailyBackup",
        scheduleExpression: cdk.aws_events.Schedule.cron({
          minute: "0",
          hour: "3",
        }), // Daily at 3 AM UTC
        deleteAfter: cdk.Duration.days(30), // Keep backups for 30 days
      })
    );

    // Outputs
    new cdk.CfnOutput(this, "DBInstanceEndpoint", {
      value: dbInstance.dbInstanceEndpointAddress,
    });

    new cdk.CfnOutput(this, "DBSecretARN", {
      value: dbCredentialsSecret.secretArn,
    });
  }
}
