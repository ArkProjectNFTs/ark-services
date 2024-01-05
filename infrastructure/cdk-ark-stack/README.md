# AWS CDK Deployment with GitHub Actions

## Overview

The AWS CDK script provided defines how the `ArkStack` should be deployed based on different contexts. It utilizes environment variables and passed-in context from the CLI to make decisions on deployment criteria. 

When integrating with GitHub Actions, these context parameters and environment variables can be set based on the specifics of the GitHub event triggering the workflow.

## CDK Script Breakdown

### Configuration & Context

1. **Environment Setup**: The script begins by loading environment variables using the `dotenv` package.
2. **Branch Information**:
   - `baseBranch`: The branch from which the action was triggered.
   - `targetBranch`: The branch targeted by a Pull Request.
3. **Flags**:
   - `stage`: production or staging
   - `isPullRequest`: Determines if the deployment is due to a PR event.
   - `prNumber`: Pull request number, useful for stack naming.

### Deployment Logic

- **Pull Request Checks**: 
  - PRs can only target the `main` branch for deployments.
  - Deployments for PRs get a stack name suffix like `pr-<prNumber>`.

- **Branch/Release Checks**: 
  - Direct deployments are allowed only on the `main` branch, releases, or pull requests. 
  - For a release, the suffix `production` is used.
  - For the `main` branch (which is not a PR), the suffix `staging` is used.

- **Stack Naming**:
  - Based on the aforementioned checks, a suffix is determined for the stack name.
  - The final stack name becomes `ArkStack-<determined-suffix>`.

### Stack Deployment

- The `ArkStack` is then instantiated and set up for deployment.
- The stack's environment is set using the AWS account ID and region from the environment variables.
- Additional metadata, like the branch and whether it's a PR, is also passed to the stack.

## Integration with GitHub Actions

With the script expecting certain context parameters and environment variables, the GitHub Actions CI/CD workflow will need to be set up in a way that it provides the required context to the CDK script. This can be achieved by:

1. **Setting Environment Variables**: 
   - Use the `env` context in GitHub Actions to set environment variables like `AWS_ACCOUNT_ID`, `AWS_REGION`, `BRANCH`, etc.
   - These environment variables can be derived from the `github` context available in GitHub Actions.

2. **Passing Context to CDK**:
   - When invoking `cdk deploy` or similar CDK CLI commands in the GitHub Actions workflow, use the `--context` flag to pass in required context like `stage`, etc.

### Example GitHub Action Step

```yaml
- name: Deploy CDK Stack
  run: |
    cdk deploy --context stage=staging --context
  env:
    AWS_ACCOUNT_ID: ${{ secrets.AWS_ACCOUNT_ID }}
    AWS_REGION: us-west-2
    IS_RELEASE: ${{ github.event_name == 'release' }}
    IS_PULL_REQUEST: ${{ github.event_name == 'pull_request' }}
    PR_NUMBER: ${{ github.event.pull_request.number }}
```

## Useful commands

* `npm run build`   compile typescript to js
* `npm run watch`   watch for changes and compile
* `npm run test`    perform the jest unit tests
* `cdk deploy`      deploy this stack to your default AWS account/region
* `cdk diff`        compare deployed stack with current state
* `cdk synth`       emits the synthesized CloudFormation template
