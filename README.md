# Simple CLI App

Simple is a Rust-based cli tool for processing cryptocurrency trade data, and performs different operations based on user-defined modes. The application is designed to cache real-time data or read previously cached data from JSON files.

## Installation
```
cargo build
```


## Usage 

+ #### Cache mode
    **Without Distributed Instances**
    ```
    cargo run -- --mode=cache --times=<time> 
    ```
     **Or**
    ```
    cargo run -- -m cache -t <time> 
    ```
    **With Distributed Instances**
    ```
    cargo run -- --mode=cache --times=<time> --client=<client instance>
    ```
    **Or**
    ```
    cargo run -- -m cache -t <time> -c <client instance>
    ```

+ #### Read mode
```
cargo run -- --mode=read
```
**Or**

```
cargo run -- -m read
```
***
## TODO
+ [ ] Signatures to each process and the aggregator checking them before aggregation.
+ [ ] Adding more tokens for get data, currently only working for btc, won't much hard.
+ [ ] Adding tests

