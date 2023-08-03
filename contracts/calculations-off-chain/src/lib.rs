#![allow(clippy::all)]

/// Returns the number of loop_iterations that were executed.
///
/// Adapted from https://github.com/aurora-is-near/aurora-engine/pull/463/files#diff-329eddf8a5c2ec43fd7d007e4716049c44f23129c2d1bb9a6d81da2f1efb51ae
#[no_mangle]
pub unsafe fn cpu_ram_soak(loop_limit: u32) -> u32 {
    let mut buf = [0u8; 100 * 1024];
    let len = buf.len();
    let mut counter = 0;

    // Convert to `usize` to use `i` below as array index. The conversion is expected to succeed for
    // target `wasm32`.
    let loop_limit = usize::try_from(loop_limit).expect("loop_limit should convert to usize");

    for i in 0..loop_limit {
        let j = (i * 7 + len / 2) % len;
        let k = (i * 3) % len;
        let tmp = buf[k];
        buf[k] = buf[j];
        buf[j] = tmp;
        counter += 1;
    }
    counter
}
