# ark-services 💠

## Pending blocks

### Install

To run the indexer locally, you need cargo-lambda. The pending blocks indexer will invoke the lambda-block-indexer lambda for each block.

```
brew tap cargo-lambda/cargo-lambda
brew install cargo-lambda
```

### Launch indexer

```
cd ark-lambdas/lambda-block-indexer
cargo lambda watch
RUST_LOG=info cargo run -p ark-indexer
```
