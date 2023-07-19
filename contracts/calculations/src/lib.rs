#![allow(clippy::all)]

use std::mem::size_of;

// Host functions used in the contract. When interpreting this contract's wasm, they must be
// provided by the interpreter.
//
// We should call at least one host function to verify providing and calling host functions works
// with interpreted wasm.
#[allow(unused)]
extern "C" {
    fn input(register_id: u64);
    fn log_utf8(len: u64, ptr: u64);
    fn read_register(register_id: u64, ptr: u64);
    fn register_len(register_id: u64) -> u64;
}

/// Expected input is the number of loop iterations encoded as `le` bytes of an `u32`.
///
/// Working with raw byte input instead of (de)serialization libraries to avoid their overhead
/// affecting benchmarks.
///
/// Adapted from https://github.com/aurora-is-near/aurora-engine/pull/463/files#diff-329eddf8a5c2ec43fd7d007e4716049c44f23129c2d1bb9a6d81da2f1efb51ae
#[no_mangle]
pub unsafe fn cpu_ram_soak() {
    // Get the number of loop iterations from input.
    let input_register = 0;
    input(input_register);
    assert_eq!(
        size_of::<u32>(),
        register_len(input_register).try_into().unwrap(),
        "unexpected input length"
    );
    let loop_limit_bytes = [0; size_of::<u32>()];
    read_register(input_register, loop_limit_bytes.as_ptr() as u64);
    let loop_limit: usize = u32::from_le_bytes(loop_limit_bytes).try_into().unwrap();

    let mut buf = [0u8; 100 * 1024];
    let len = buf.len();
    let mut counter = 0;
    for i in 0..loop_limit {
        let j = (i * 7 + len / 2) % len;
        let k = (i * 3) % len;
        let tmp = buf[k];
        buf[k] = buf[j];
        buf[j] = tmp;
        counter += 1;
    }
    let msg = format!("Done {} iterations!", counter);
    log_utf8(msg.len() as u64, msg.as_ptr() as u64);
}
