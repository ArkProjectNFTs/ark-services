# Performance task to do and code review to fix

## About 
This document aims to product a better performance inside ark-marketplace-cron project. 
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

### - main.rs - 116-139 -  Avoid using string Type in struct prefer using &str (will lead to pointers and zero-cpoy), use lifetime parameters to ensure value is valid as long we need.
#### Report
When using string, as long as the value is moved to an another context, it will lead automaticaly to a copy whereas &str will only pass the reference.
DUPLICATED CODE DETECTED / Should use a common lib between project
#### Reference 
https://stackoverflow.com/questions/24158114/what-are-the-differences-between-rusts-string-and-str
https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html
#### Fix - Implementation
```rust
struct DatabaseCredentials<'a> {
    username: &'a str,
    password: &'a str,
    dbname: &'a str,
    port: u16,
    host: &'a str,
}
```

### - token.rs - 58 - Vec of Sting could be VEC of &str ( pointers of string )
#### Report
String = vec<u8> => will lead automaticaly to a copy when used by for loop, iter , print , format etc ... everytime when a context will need the ownership
#### Reference 
#### Fix - Implementation
The line 60 before the collect should also beeing change
```rust
change vec<String> into vec<&str> 
```

### - token.rs - 67-105 - Wrong place of code / Double For Loop / clone that can be avoid 
#### Report
The clone line 67 could be avoid 
Double for loop for the same vector
Code totaly executing 2 tipe ref and deref of each variable of the loop
#### Reference 
https://docs.rs/rayon/latest/rayon/iter/index.html
#### Fix - Implementation
Simply remove the .clone from collection
The clone line 67 could be avoid 
the for loop could totaly be inside the first for loop and the code to clean with date could be declared before. 
Plus the fact that here we can use par_iter from rayon instead of for loop in order to run queries concurently.

### - token.rs - 76-51 - 261- use const instead of let when using string literal 
#### Report
When using let it drive to use memory address at launch, the const variable will inline the string into the final program opcode and will drive to no memory call / usage / ref & deref -> So lead to better performance
#### Reference 
#### Fix - Implementation
Best fast fix -> write const instead of let 
Best Pragmatic fix -> put all the query inside a const file ( query.rs for example ) => Will lead to easly find dobles inside queries 

### - token.rs - 229-230 - use of haset of string + into_iter
#### Report
Usage of into_iter()
Hashset of &str
#### Reference 
#### Fix - Implementation
Use iter() instead of into_iter()
replace Hashsetof String to hashset<&'str>

### - token.rs - 261 - defined array should stay as array
#### Report
usage of a vec instead of an array ( list of contracts defined )
#### Reference 
#### Fix - Implementation
```rust
const collections_to_cache: [&str,3] = ["0x05dbdedc203e92749e2e746e2d40a768d966bd243df04a6b712e222bc040a9af",
    "0x076503062d78f4481be03c9145022d6a4a71ec0719aa07756f79a2384dc7ef16",
    "0x0169e971d146ccf8f5e88f2b12e2e6099663fb56e42573479f2aee93309982f8"]
```


## Conclusions
#### Report
The project need work on the side of architecture of code.
This optimization could be done in a range of 2 - 3 days max depending on the 
In terms of architecture a lot of improvement could be done .
I'm wondering whill the postgresql connection pool storage is limited to 1 connection.
types.rs | aws_s3_file_manager.rs | metadata_storage.rs should be in respective directory ie: example