#![allow(clippy::all)]

use wasmi::{Caller, Engine, Extern, Func, Linker, Module, Store};

// Host functions used in the contract.
#[allow(unused)]
extern "C" {
    fn input(register_id: u64);
    fn log_utf8(len: u64, ptr: u64);
    fn read_register(register_id: u64, ptr: u64);
    fn register_len(register_id: u64) -> u64;
}

/// Expects the bytes of compiled contract `contracts/calculations` as input.
#[no_mangle]
pub unsafe fn cpu_ram_soak() {
    // Get wasm bytecode from input.
    const INPUT_REGISTER: u64 = 0;
    input(INPUT_REGISTER as u64);
    let wasm = get_register_data(INPUT_REGISTER);
    assert!(wasm.len() > 0, "Input should be wasm bytecode.");

    // Set up the interpreter.
    let engine = Engine::default();
    let module = Module::new(&engine, &mut &wasm[..]).expect("should create `Module`");
    type HostState = (); // a type is required, but for now we don't need state
    let mut store = Store::new(&engine, ());

    // Any host functions used in the interpreted wasm need to be available as imports in the
    // interpreter. Define the functions here.

    // Proxies Near's `log_utf` host function.
    let host_fn_log_utf8 = Func::wrap(
        &mut store,
        |caller: Caller<'_, HostState>, len: i64, ptr: i64| {
            // Read data from guest memory.
            //
            // When compiling a contract written in Rust to `wasm32-unknown-unknown`, memory is
            // exported under the name `memory`.
            let memory = caller
                .get_export("memory")
                .and_then(Extern::into_memory)
                .expect("should export memory");
            let mut msg = vec![0; len.try_into().unwrap()];
            memory
                .read(caller, ptr.try_into().unwrap(), &mut msg)
                .expect("should read from interpreter's memory");

            log_utf8(msg.len() as u64, msg.as_ptr() as u64);
        },
    );

    // Create a `Linker` to link the module's imports and exports.
    let mut linker = <Linker<HostState>>::new(&engine);
    linker
        .define("env", "log_utf8", host_fn_log_utf8)
        .expect("should link host fn `log_utf8`");

    // Instantiate the module. Before using the instance, `wasmi` requires calling `start()`.
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("should instantiate the module")
        .start(&mut store)
        .expect("should start the module");

    // Call the exported function.
    let cpu_ram_soak = instance
        .get_typed_func::<(), ()>(&store, "cpu_ram_soak")
        .expect("should get the exported function");
    cpu_ram_soak
        .call(&mut store, ())
        .expect("should call the exported function")
}

unsafe fn get_register_data(register_id: u64) -> Vec<u8> {
    let result = vec![0; register_len(register_id).try_into().unwrap()];
    read_register(register_id, result.as_ptr() as u64);
    result
}
