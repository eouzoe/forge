//! Fuzz target: SHA-256 hash computation.
//!
//! Verifies that `compute_hash` never panics on arbitrary input and always
//! produces a 64-character hex string.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let hash = forge_executor::compute_hash(data, &[]);
    let hex = hash.to_string();
    assert_eq!(hex.len(), 64, "SHA-256 hex must always be 64 chars");
    assert!(
        hex.chars().all(|c| c.is_ascii_hexdigit()),
        "SHA-256 hex must contain only hex digits"
    );
});
