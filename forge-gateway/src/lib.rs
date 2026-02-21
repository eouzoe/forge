//! HTTP API gateway for the Forge deterministic execution fabric.
//!
//! Exposes sandbox lifecycle and code execution endpoints for the
//! Deepractice `SandboX` isolator integration.
//!
//! See `docs/ARCHITECTURE.md` §4 for design rationale.

#![warn(clippy::pedantic)]
#![deny(clippy::unwrap_used)]

pub mod error;
pub mod pool;
pub mod routes;
