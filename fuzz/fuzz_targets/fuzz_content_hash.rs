//! Fuzz target: `ContentHash` Display and round-trip serialization.
//!
//! Verifies that arbitrary 32-byte inputs produce valid hex strings
//! and that JSON serialization never panics.

#![no_main]

use forge_core::ContentHash;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Only process inputs that are exactly 32 bytes (SHA-256 size).
    if data.len() != 32 {
        return;
    }
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(data);

    let hash = ContentHash::new(bytes);

    // Display must not panic and must produce 64 hex chars.
    let hex = hash.to_string();
    assert_eq!(hex.len(), 64, "ContentHash Display must produce 64 hex chars");

    // JSON round-trip must not panic.
    let json = serde_json::to_string(&hash).expect("ContentHash serialization must not fail");
    let _: ContentHash =
        serde_json::from_str(&json).expect("ContentHash deserialization must not fail");
});
