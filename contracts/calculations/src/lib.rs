#![allow(clippy::all)]

// Host functions used in the contract. When interpreting this contract's wasm, they must be
// provided by the interpreter.
//
// We should call at least one host function to verify providing and calling host functions works
// with interpreted wasm.
#[allow(unused)]
extern "C" {
    fn log_utf8(len: u64, ptr: u64);
}

// Adapted from https://github.com/aurora-is-near/aurora-engine/pull/463/files#diff-329eddf8a5c2ec43fd7d007e4716049c44f23129c2d1bb9a6d81da2f1efb51ae
#[no_mangle]
pub unsafe fn cpu_ram_soak() {
    // Use a hardcoded loop limit to avoid depending on input. This allows interpreting this
    // contract without having to read input data within the interpreter.
    let loop_limit = 100;

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
