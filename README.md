# About

Execute `contracts/calculations` as Near contract and interpret it inside another Near contract by running the following command. Gas usage will be printed to standard output and the calculations are repeated `loop-limit` times.

# Usage

```
cargo run -- --help
```

# Results

```
$ cargo run -- --loop-limit 1,100,1000,10000,20000

+-------------------+----------------+------------------+-------------------+--------------------+--------------------+
| exec_mode         | loop_limit = 1 | loop_limit = 100 | loop_limit = 1000 | loop_limit = 10000 | loop_limit = 20000 |
+=====================================================================================================================+
| native            | 2.6680 TGas    | 2.6729 TGas      | 2.7117 TGas       | 3.0971 TGas        | 3.5250 TGas        |
|-------------------+----------------+------------------+-------------------+--------------------+--------------------|
| wasmi interpreter | 73.6618 TGas   | 74.7821 TGas     | 84.8506 TGas      | 185.4982 TGas      | 297.3189 TGas      |
+-------------------+----------------+------------------+-------------------+--------------------+--------------------+
```

Observations:

- Interpreter setup for `wasmi` roughly costs 70 TGas (compare gas usage for `loop_limit = 1`).
    - This can be a deal breaker if an Ethereum XCC is to be executed in a separate interpreter instance.
- Increasing the `loop_limit` from 1 to 20_000 only slightly increases `native` gas usage. For `wasmi interpreter`, however, it drives gas usage up to the limit per transaction of 300 TGas.

# Log noise

The following noise in logs is due to [this issue](https://github.com/near/workspaces-rs/issues/272):

```
Updated the logging layer according to `log_config.json`
```
