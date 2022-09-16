# TVM
TON Virtual Machine implementation

## Prerequisites

https://www.rust-lang.org/en-US/install.html

## To Build & Run:

```
cargo build
cargo run
```

## To Test:
```
cargo test
```
## Features:
`--features`
`ci_run` - run long tests
`fift_check` - check test results using fift binaries should be near test executable
`log_file` - ouput log to file
`verbose` - show execution process, don't forget to call `logger::init()`
`use_test_framework` - use tvm's test framework

## Verbose output
We can get verbose information about TVM execution, such as primitive name with parameters, stack dump and values of control registers after each executed command.
Logging can work in some ways:
1. We want to get verbose output of one broken own test in TVM. Run this test with key --features verbose
`cargo test --test test_gas buygas_normal --features verbose`

2. We want to get verbose output of TVM execution wich is inluded as library to other application (for example node)
In application use log4rs crate init procedure `log4rs::init_file` or use predefined set from TVM calling `tvm::init_full` with relative path to config file.
Available targets in logging are: `compile` - trace compile process and `tvm` - trace execution process
The level of tracing: trace and higher

3. Old way to do nothing new but no control

## Compile smart contract:

After build project you can use **compile** util from `target/release/compile` or `target/debug/compile` for compile your contract.

Commands (by unix example):
- Compile contract
  `./compile your_bytecode_file your_cells_file`
- Get help
  `./compile --help`

## Execute smart contract:

After build project you can use **execute** util from `target/release/execute` or `target/debug/execute` for execute your contract.

Commands (by unix example):
- Execute contract
  `./execute your_contract_file`
  - Execute contract with stack items (strings)
    `./execute your_contract_file --params stack-items`
- Get help
  `./execute --help`
