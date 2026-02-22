//! Fuzz target: serial console output parser.
//!
//! Feeds arbitrary bytes through the base64 decoder used by the serial output
//! parser. The decoder must never panic regardless of input.
#![no_main]

use base64::Engine as _;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Test that base64 decoding of arbitrary input never panics.
    // This exercises the inner loop of parse_execution_output.
    let _ = base64::engine::general_purpose::STANDARD.decode(data);

    // Also verify that compute_hash handles arbitrary bytes without panicking.
    let hash = forge_executor::compute_hash(data, data);
    let hex = hash.to_string();
    assert_eq!(hex.len(), 64);
});
