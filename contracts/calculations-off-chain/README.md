A wasm module to run off-chain executing similar calculations as `contracts/calculations`.

To make this as simple as possible the module has the following characteristics:

- No host functions: avoid having to implement them for every runtime/interpreter.
