import * as cdk from "aws-cdk-lib";
import { aws_ecr as ecr } from "aws-cdk-lib";
import { Construct } from "constructs";

export class ECRDeploymentStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // Create an ECR Repository
    const ecrRepository = new ecr.Repository(this, "ark-project-repo", {
      repositoryName: "ark-project-repo",
    });

    // Set the deletion policy to DELETE
    const ecrRepositoryResource = ecrRepository.node
      .defaultChild as cdk.CfnResource;
    ecrRepositoryResource.cfnOptions.deletionPolicy =
      cdk.CfnDeletionPolicy.DELETE;

    // Output the ECR Repository URL for further use
    new cdk.CfnOutput(this, "ECRRepositoryURL", {
      value: ecrRepository.repositoryUri,
      description: "The URI of the ECR Repository",
    });
  }
}
