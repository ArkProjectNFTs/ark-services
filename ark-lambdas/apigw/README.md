# Lambda HTTP behing Api Gateway (apigw)

This folder contains all the lambdas that must be deployed behing the Api Gateway.

Those lambdas are `HTTP` lambdas, and the argument `event` that is passed to them
is a [Request from the lambda_http](https://docs.rs/lambda_http/0.8.1/lambda_http/type.Request.html).

You have examples of how lambdas can be built on the [aws official repo](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples).

## Shared code

Shared code must be placed in the dedicated crates.

First, `lambdas/lambda-dynamo-common` is the crate related to any dynamoDB requests, centralized for all the lambdas.

Then, `lambdas/apigw/lambda-http-common` contains any code that is very specific to the HTTP protocol that may be used by any lambda called from HTTP (parsing headers, query string params, the body etc...).

## How to create a new lambda

To create a new lambda, first ensure that you've `cargo lambda` installed. [Instructions here](https://www.cargo-lambda.info/guide/installation.html).

Each lambda must live in a **separate crate**.

```bash
# Go into the lambdas/apigw to create the lambda.
cd lambdas/apigw/

# Create the lambda crate with the prefix `lambda`.
cargo lambda new lambda-<name_of_the_lambda>

# Answer yes to `HTTP` lambda on creating the lambda.

# Add the lambda as a member of to the top level workspace.
cd ark-services/
```

```toml
[workspace]
members = [
  ...
  "lambdas/apigw/lambda-<name_of_the_lambda>",
]
```

## Implement a lambda

To allow easy testing with mocking, the lambda must be built with a context
that bundle all the dependencies on external trait.

Take the example of `lambda-get-contract`:

```rust
struct Ctx<P> {
    client: DynamoClient,
    provider: P,
}

async fn function_handler<P: ArkContractProvider<Client = DynamoClient>>(
    ctx: &Ctx<P>,
    event: Request,
) -> Result<Response<Body>, Error> {
...
}
```

The `Ctx` is only generic for provider, which is a trait required in this specific
case to fetch data related to a contract.

The `Ctx` does take ownership of the client and the provider, as the lambda lifecycle is short, and the init is done only once.

There is no need to internalize the `client` inside the `provider` as we want to keep the best flexibility to implement the lambda functionalities.

Depending on your need, don't hesitate to populate the `Ctx` accordingly and adjust the generic types of the lambda function.

## Build a lambda

To build a specific lambda, being at the root of `ark-services`, you can run:

```bash
cargo build -p lambda-<name_of_the_lambda>
```

## Test the lambda

To test the lambda, there are two solutions:
* An interactive solution using `cargo lambda watch --env-file .aws` where you can invoke the lambda sending HTTP request to the local server started by `cargo lambda`. This is useful to test the lambda manually on the backend service it is connected to without having to deploy it.

  Natively, the lambda does not support HTTP request!! This is the Api Gateway that is converting for the lambda to be able to consume the request. So, to test a request with a body + path + query parameters etc.., we must use a special JSON file with the expected format by the lambda runtime.
  
  You can find a example of this file into the `lambda-get-contract/data-files/http.json`. So we can configure several files, with good and bad data, or with various items we want to check from the DB.

  To test with watch, open a first terminal and run `cargo lambda watch`. In an other terminal, you can then do:
  ```bash
  # In this example, we pass the a payload only!
  cargo lambda invoke lambda-<name_of_the_lambda> --data-ascii "{ \"name\": \"everai\" }"
  
  # Or use a JSON file with the body direclty.
  cargo lambda invoke lambda-get-contract --data-file ./ark-lambdas/apigw/lambda-get-contract/data-files/existing.json
  ```
  You can check the documentation of [cargo lambda invoke](https://www.cargo-lambda.info/commands/invoke.html).
  
  The first time you will call your lambda, `cargo lambda` must build the binary, so it's totally normal that the first invokation is very slow. Only due to `cargo lambda`, not how it will be once deployed.


* A `cargo test` fashion, where you can write the tests in `rust` to test your lambda. This is the preferred way to test the lambda into the CI.

  With this method, you can directly test the lambda code inside the `rust` code.
  
  Some how-to:
  
  ```rust
  // Build a lambda with query string parameters:
  let mut params = HashMap::new();
  mocked.insert("name".to_string(), "everai".to_string());
  let req = Request::default().with_query_string_parameters(params.clone());

  let rsp = name_of_the_handler(req).await.expect("failed to handle request");
  
  assert_eq!(rsp.status(), 200);
  ...
  // Add more verification here if needed.
  ```
  
  ```rust
  // Build a lambda with a body.
  let body = r#"{ "name": "everay" }"#;

  let req = http::Request::builder()
      .uri("https://api.com")
      .header("Content-Type", "application/json")
      .body(Body::Text(body.into())).expect("fail building request");
      
  let rsp = name_of_the_handler(req).await.expect("failed to handle request");
  assert_eq!(rsp.status(), 200);
  ```
  
  Using this approach, we can then also mock the storage exactly as it was done
  for unit testing in `pontos`.
  
  The main purpose is to test the input request:
      * The path may be checked
      * Query string parameters if any
      * The expected body if any
      * The headers if we need some extra validation

  To run the tests, simply use:
  ```bash
  cargo test -p lambda-<name_of_the_lambda>
  ```
