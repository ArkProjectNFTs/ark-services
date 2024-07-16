# Performance task to do and code review to fix

## About 
This document aims to product a better performance inside ark-marketplace-api project. 
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

### - db/db_access.rs - 34-35-635-636 -  String instead of &str
#### Report
Usage of String instead of &str -> will lead to copy
#### Fix - Install
Change String declaration to &str

### - db/db_access.rs - 476 -  use const instead of let
#### Report
Some variable are declared with let in order to declare string literals, use const will lead to inlining insted of use of memory
#### Fix - Install
change let to const

### - db/db_access.rs - 476 -  use const instead of let
#### Report
Some variable are declared with let in order to declare string literals, use const will lead to inlining insted of use of memory
#### Fix - Install
change let to const

### - handler/collection_handler.rs - 64 -  mutex not free 
#### Report
the mutext is lock until the scope change
#### Fix - Install
assign the mutex to a variable and then drop the mutex

### - models/collection models/token - ALL -  Usage of string instead of &str
#### Report
string should be change to &str ( all the Struct should use lifetime parameters)
#### Fix - Install
change string to &str 
Use lifetime parameters

### - utils/currency_utils - ALL -  currency_address not used
#### Report
The variable should not be declared if it's not used
#### Fix - Install
remove the variable declaration

### - utils/currency_utils - 12-13-14 -  unwrap use instead of match
#### Report
unwrap usage will lead to not predictable result => evaluation of the option without a matching patern => will drive a wait instead of L2 cache prefetching
#### Fix - Install
change unwrap to if let or match

### - utils/currency_utils - 14 -  clone on floor_price
#### Report
unused clone, as long as the data will not be mutate this clone should be avoid (zero-copy)
#### Fix - Install
remove .clone() declaration

## Conclusions
#### Report
The project needs some work but seems to have less work than the other microservices.
There is mainly optimisations possible inside caching of queries. 
