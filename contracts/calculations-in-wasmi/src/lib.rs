#![allow(clippy::all)]

use std::collections::HashMap;
use std::mem::size_of;

use wasmi::{Caller, Engine, Extern, Func, Linker, Memory, Module, Store};

// Host functions used in the contract.
#[allow(unused)]
extern "C" {
    fn input(register_id: u64);
    fn log_utf8(len: u64, ptr: u64);
    fn read_register(register_id: u64, ptr: u64);
    fn register_len(register_id: u64) -> u64;
}

/// Host state related to the execution of wasm bytecode.
///
/// Instances are throwaway objects that must be used only for the execution of a single exported
/// function, since the fields contain data specific to a `FunctionCallAction`.
struct HostState {
    /// The data the guest sees as its input and which it can access with the `input` host function.
    ///
    /// This data should live in `HostState` to avoid lifetime and ownership issues due to
    /// constraints on host functions in `wasmi`, (see [`wasmi::Func::wrap`]).
    input: Vec<u8>,
    /// Some of these host functions read/write registers. We use a map to virtualize registers used
    /// by the guest. Since the map only exists in memory, using `std::collections` is fine. If it
    /// were written to storage, `near_sdk::collections` should be used.
    registers: HashMap<u64, Vec<u8>>,
}

impl HostState {
    fn new(input: Vec<u8>) -> Self {
        Self {
            input,
            registers: HashMap::new(),
        }
    }

    fn get_register_data(&self, register_id: u64) -> Vec<u8> {
        match self.registers.get(&register_id) {
            Some(data) => data.clone(),
            None => vec![],
        }
    }
}

/// Expects input bytes correpsonding to:
///
/// - [0..4]: `loop_limit` to be passed to the interpreted wasm as `le_bytes` of an `u32`
/// - [5..]: wasm of `contracts/calculations`
///
/// Working with raw byte input instead of (de)serialization libraries to avoid their overhead
/// affecting benchmarks.
#[no_mangle]
pub unsafe fn cpu_ram_soak() {
    // Get input data.
    const INPUT_REGISTER: u64 = 0;
    input(INPUT_REGISTER as u64);
    let mut input_data = get_register_data(INPUT_REGISTER);
    let min_input_len = size_of::<u32>() + 1; // + 1 to verify wasm bytecode is not empty
    assert!(input_data.len() >= min_input_len, "unexpected input");

    // Split input data. Cloning _small_ data to simplify ownership.
    let loop_limit_bytes = input_data.drain(..4).collect::<Vec<u8>>().clone();
    let wasm = input_data;

    // Set up host state. We want the guest to see loop_limit_bytes as its input.
    let host_state = HostState::new(loop_limit_bytes);

    // Set up the interpreter.
    let engine = Engine::default();
    let module = Module::new(&engine, &mut &wasm[..]).expect("should create `Module`");
    let mut store = Store::new(&engine, host_state);

    // Any host functions used in the interpreted wasm need to be available as imports in the
    // interpreter. Define the functions here.

    // Writes input for `wasm` into the specified virtual guest register.
    let host_fn_input = Func::wrap(
        &mut store,
        |mut caller: Caller<'_, HostState>, register_id: i64| {
            let input = caller.data().input.clone();
            caller
                .data_mut()
                .registers
                .insert(register_id as u64, input);
        },
    );

    // Allow the guest to write the data stored in a virtualized register to memory starting at
    // `ptr`.
    let host_fn_read_register = Func::wrap(
        &mut store,
        |caller: Caller<'_, HostState>, register_id: i64, ptr: i64| {
            let data = caller
                .data()
                .get_register_data(register_id.try_into().unwrap());
            let memory = get_exported_memory(&caller);
            memory
                .write(caller, ptr.try_into().unwrap(), &data)
                .expect("should write memory");
        },
    );

    // Allows the guest to get the byte-size of the data stored in `register_id`. Returns 0 if the
    // register is empty.
    let host_fn_register_len = Func::wrap(
        &mut store,
        |caller: Caller<'_, HostState>, register_id: i64| {
            let data = caller
                .data()
                .get_register_data(register_id.try_into().unwrap());
            data.len() as i64
        },
    );

    // Proxies Near's `log_utf` host function.
    let host_fn_log_utf8 = Func::wrap(
        &mut store,
        |caller: Caller<'_, HostState>, len: i64, ptr: i64| {
            let mut msg = vec![0; len.try_into().unwrap()];
            let memory = get_exported_memory(&caller);
            memory
                .read(caller, ptr.try_into().unwrap(), &mut msg)
                .expect("should read from interpreter's memory");

            log_utf8(msg.len() as u64, msg.as_ptr() as u64);
        },
    );

    // Create a `Linker` to link the module's imports and exports.
    let mut linker = <Linker<HostState>>::new(&engine);
    linker.define("env", "input", host_fn_input).unwrap();
    linker.define("env", "log_utf8", host_fn_log_utf8).unwrap();
    linker
        .define("env", "read_register", host_fn_read_register)
        .unwrap();
    linker
        .define("env", "register_len", host_fn_register_len)
        .unwrap();

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

/// Gets the memory that is exported by a wasm module written in Rust and compiled to
/// `wasm32-unknown-unknown`.
fn get_exported_memory(caller: &Caller<'_, HostState>) -> Memory {
    caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .expect("should export memory")
}

unsafe fn get_register_data(register_id: u64) -> Vec<u8> {
    let result = vec![0; register_len(register_id).try_into().unwrap()];
    read_register(register_id, result.as_ptr() as u64);
    result
}
