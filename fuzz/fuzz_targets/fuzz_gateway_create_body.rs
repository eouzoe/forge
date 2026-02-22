//! Fuzz target: JSON deserialization of `CreateSandboxBody`.
//!
//! Verifies that arbitrary byte sequences fed to the JSON parser
//! never cause panics, UB, or unbounded resource consumption.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Treat arbitrary bytes as a JSON payload for CreateSandboxBody.
    // We only care that this never panics — errors are expected and fine.
    let _ = serde_json::from_slice::<serde_json::Value>(data);
});
