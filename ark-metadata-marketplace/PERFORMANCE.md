# Performance task to do and code review to fix

## About 
This document aims to product a better performance inside ark-metatada-marketplace project. 
All of the recomandation here should lead to performance improvements reducing lattency issues and ingestion congestion. 

## Current State

![Flamegraph](./flamegraph.svg)

## Table of Content 

### - main.rs - l22 -  Change The Memory default Alocator 
#### Report
Using the jmaloc leads to better performance in our case than the use of the default glibc allocator (that rust use by default)
#### Reference 
https://github.com/rust-lang/rust-analyzer/issues/1441
#### Fix - Install
```bash
cargo add tikv_jemallocator
```
#### Fix - Implementation
Before the main function 
```rust
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
```

### - aws_s3_file_manager.rs - L12 -  change struct to lifetimed struct ( zero-copy )
#### Report
Bucket name could be a &str instead, 
#### Fix - Implementation
Before the main function 
```rust
pub struct AWSFileManager<'a> {
    bucket_name: &'a str,
}
```

### - aws_s3_file_manager.rs - L34-39-90 -  Use &str instead of string
#### Report
Use &str instead of String 
#### Fix - Implementation
// TDB


### - main.rs - L22-39 -  change struct to lifetimed stuct ( zero-copy )
#### Report
Use &str instead of String 
#### Fix - Implementation
```rust
struct Config<'a> {
    bucket_name: &'a str,
    rpc_url: &'a str,
    ipfs_timeout_duration: Duration,
    loop_delay_duration: Duration,
    ipfs_gateway_uri: &'a str,
    filter: Option<(&'a str,  &'a str)>,
    refresh_contract_metadata: bool,
}
....
```

### - main.rs - L66-96  -  Avoid loading env while in runtime ( pefer load .env then store it into Config Variable )
#### Report
Even if it's an initial step, load env each time you need a value leads to take 4ns reading .env file each time you need to read env.
#### Reference 
No Refs
#### Fix - Implementation
Use Envy or any kind of things to load bulk environement data to a config struct ( data latency access optimisation )
ie:
ARK_METADATA_MARKETPLACE_AWS_KEY
ARK_METADATA_MARKETPLACE_AWS_SECRET
ARK_METADATA_MARKETPLACE_DATABASE_USER
....
```rust 
    match envy::prefixed("ARK_METADATA_MARKETPLACE_").from_env::<Config>() {
        Ok(config) => {
            ...
        },
        Err(error) => panic!("{:#?}", error),
    }
```

### - main.rs - L41-64  -  The function should return a &str instead of string
#### Report
Avoid returning String while the aim is to use it inside a fmt function ( will drive to create a copy on fmt format function)
#### Reference 
No Refs
#### Fix - Implementation
TBD

### - main.rs - L41-64  -  The function should return a &str instead of string
#### Report
Avoid returning String while the aim is to use it inside a fmt function ( will drive to create a copy on fmt format function)
#### Reference 
No Refs
#### Fix - Implementation
TBD

### - main.rs - L219  -  Don't use match as a wrapper function
#### Report
Using match as a wrapper leads to evaluate all the code while it goest to the error step
#### Reference 
https://doc.rust-lang.org/std/keyword.continue.html
#### Fix - Implementation
You should first match and get the data from the match -> then use continue keywords in order to shortcut the loop. 

### - main.rs - L219  -  Should be to ENV or a CONST 
#### Report
This variable is witten in clear => Will lead if it's declared multiple time to double constant allocation instead of one by the compiler
#### Reference 
NONE
#### Fix - Implementation
Change the content of those line by moving it to an env or a constant 
```rust 
// interfaces/const.rs
    const ARK_URL "https://arkproject.dev";
```
or
```rust 
// .env
    ARK_URL="https://arkproject.dev";
```

### - metadata_storage.rs - L70  -  timestamp generation on query should be avoid
#### Report
timestamp should be send by the program or being auto-generated on update ( lead to computation & evaluation on query execution)
#### Reference 
NONE
#### Fix - Implementation

### - token.rs - 68-124-159 - use const instead of let when using string literal 
#### Report
When using let it drive to use memory address at launch, the const variable will inline the string into the final program opcode and will drive to no memory call / usage / ref & deref -> So lead to better performance
#### Reference 
#### Fix - Implementation
Best fast fix -> write const instead of let 
Best Pragmatic fix -> put all the query inside a const file ( query.rs for example ) => Will lead to easly find dobles inside queries 

### - token.rs - 202 - use  of into_iter instead of iter 
#### Report
into iter usage instead of iter: into iter should be use only when consuming from iterator
#### Reference 
#### Fix - Implementation
replace into_iter() to iter()


### - ALL -  File & folder Architecture
#### Report
Clear file and fonder architecture leads this micro service to be maintain better with best practices
#### Reference 
NONE
#### Fix - Implementation
PROPOSAL 
```shell
/project
|-> /src
  |-> helpers
    mod.rs
  |-> interfaces
    mod.rs
  |-> models
    mod.rs
  |-> services
    mod.rs
    |-> restapi // only if API - endpoint where to declare actix_web
        mod.rs
        server.rs
        |-> middlwares
            token.rs
            mod.rs
    |-> store // all items to have IN memory
        mod.rs
        metadata_indexer.rs
    |-> store // all items to have IN memory
        mod.rs
        metadata_indexer.rs
    |-> worker // all things that execute periodicaly
        mod.rs
        metadata_indexer.rs
    |-> service_name (could be indexer) // all things that execute periodicaly
        mod.rs
        processor.rs
        decode.rs
        ...
  main.rs
  lib.rs
|-> /migrations 
```


## Conclusions
#### Report
The project need work on the side of architecture of code.
This optimization could be done in a range of 2 - 3 days max depending on the 
In terms of architecture a lot of improvement could be done .
I'm wondering whill the postgresql connection pool storage is limited to 1 connection.
types.rs | aws_s3_file_manager.rs | metadata_storage.rs should be in respective directory ie: example