![Ark Project](/images/arkproject.png)

# Project Structure

The **ark-services** repository is organized into several sub-modules:

### ark-dynamodb

- Description: Contains data access functions, especially queries for AWS DynamoDB.
- Utility: Enables interaction with the DynamoDB database for data retrieval and management.

### ark-indexer

- Description: Implementation of the traits of the Pontos indexer. Provides specific configuration for indexing data from NFT APIs.
- Utility: Indexes data retrieved from NFT APIs and organizes it for easy access and analysis.

### ark-indexer-admin

Description: Front-end application for a dashboard of indexed blocks and monitoring indexing tasks.
Utility: Allows users to view and monitor the indexing status in real-time.

### ark-lambdas

- Description: Contains cargo-lambda code to deploy AWS Lambda functions that correspond to API Gateway endpoints (NFT APIs).
- Utility: Facilitates the setup and management of serverless functions for processing requests to the APIs.

### ark-metadata-refresh

- Description: Background job for refreshing metadata of tokens for indexed contracts.
- Utility: Ensures that token data remains up-to-date and reflects the latest changes.

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

## Launch indexer

```
cd ark-lambdas/lambda-block-indexer
cargo lambda watch
RUST_LOG=info cargo run -p ark-indexer
```

## Indexer Monitoring

```
cd ark-indexer-admin
pnpm run dev
```
