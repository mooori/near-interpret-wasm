# Usage

Execute `contracts/calculations` as Near contract and interpret it inside another Near contract by running the following command. Gas usage will be printed to standard output and the calculations are repeated `loop-limit` times.

```
cargo run -- --loop-limit <u32>
```

# Log noise

The following noise in logs is due to [this issue](https://github.com/near/workspaces-rs/issues/272):

```
Updated the logging layer according to `log_config.json`
```
