![Ark Project](/images/arkproject.png)

# Project Structure

The **ark-services** repository is organized into several sub-modules:

### ark-indexer-admin

Description: Front-end application for a dashboard of indexed blocks and monitoring indexing tasks.
Utility: Allows users to view and monitor the indexing status in real-time.

### ark-lambdas

- Description: Contains cargo-lambda code to deploy AWS Lambda functions that correspond to API Gateway endpoints (NFT APIs).
- Utility: Facilitates the setup and management of serverless functions for processing requests to the APIs.

### ark-sqlx

- Description: Access to data and SQL queries.
- Utility: Allows manipulation and querying of SQL databases for the project.

### arkchain-indexer

- Description: Indexer for the Arkchain orderbook.
- Utility: Specific to indexing and analyzing data from the Arkchain orderbook.

## Install

To run the indexer locally, you need cargo-lambda. The pending blocks indexer will invoke the lambda-block-indexer lambda for each block.

```
brew tap cargo-lambda/cargo-lambda
brew install cargo-lambda
```
