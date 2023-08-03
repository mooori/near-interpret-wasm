# About

Comparison of wasm runtimes or interpreters by executing a module off-chain.Off-chain execution allows comparing different runtimes or interpreters directly, for instance because they cannot (easily) be embedded in Near smart contracts.

# Preparations

- Install the [`wasmi`](https://github.com/paritytech/wasmi) interpreter CLI application with `cargo install wasmi_cli`.
- [Build `iwasm`](https://github.com/bytecodealliance/wasm-micro-runtime/tree/main/product-mini), the `wamr` CLI application with `fast interpreter` enabled (the interpreter mode is set at compile time).
    - Warning: executables distributed with [release v1.2.2](https://github.com/bytecodealliance/wasm-micro-runtime/releases/tag/WAMR-1.2.2) do _not_ have `fast interpreter` enabled. Instead they have `classic interpreter` enabled.
    - In case you want to reproduce the results of `wamr_classic_interp` another `iwasm` build is required with `classic interpreter` enabled.
- Ensure [`perf`](https://perf.wiki.kernel.org/index.php/Main_Page) is available on your system.

# Results

The following table contains the average `task clock` time of interpreting `contracts/calculations-off-chain` in different interpreters respectively modes. The average is calculated over 10 runs and standard deviation is omitted since it is below 10%, which does not change the big picture here.

| loop_limit | wamr_fast_interp | wasmi    | wamr_classic_interp |
|------------|------------------|----------|---------------------|
| 100k       | 15 msec          | 12 msec  | 21 msec             |
| 10M        | 365 msec         | 729 msec | 968 msec            |
| 100M       | 3406 msec        | 6819 msec| 9663 msec           |

The commands used to obtain these numbers are contained in `Makefile`. Results are dependent on the system and its load.

## Observations

### `wasmi <= wamr_fast_interp`` for low `loop_limit`

This might be explained by the `fast interpreter`'s refactorings that are made to improve interpreter performance. They are carried out at load time ([ref](https://www.intel.com/content/www/us/en/developer/articles/technical/webassembly-interpreter-design-wasm-micro-runtime.html)). For a low `loop_limit` the costs of these refactorings might outweight their benefits.

### `wamr_fast_interp vs wasmi` speedup ~ x2 for high `loop_limit`

Given the optimizations of `wamr`'s `fast interpreter` a speedup in comparison to `wasmi` was expected.

## Conclusions

Assuming the `wamr fast interpreter` speedup over `wasmi` roughly remains at the same factor of ~2 when embedded in Near contracts, trying to make it work with `wamr` seems not promising. The gas costs of using `wamr` are likely to be too high for Aurora's use case. An overview of the difficulties related to embedding `wamr` in a Near contract are described [here](https://github.com/near/NEPs/pull/481#issuecomment-1655562009).
