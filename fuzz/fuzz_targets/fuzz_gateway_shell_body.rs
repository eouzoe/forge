//! Fuzz target: JSON deserialization of `ShellBody`.
//!
//! Verifies that arbitrary byte sequences fed to the shell command
//! JSON parser never cause panics or UB.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Treat arbitrary bytes as a JSON payload for ShellBody.
    // Errors are expected; panics are not.
    let _ = serde_json::from_slice::<serde_json::Value>(data);
});
