//! Audit engine for verifying deterministic execution results in Forge.
//!
//! Validates execution records, computes trust scores, and enforces
//! the trust level policy for block composition.

#![warn(clippy::pedantic)]
#![deny(clippy::unwrap_used)]
