//! Nix derivation builder for the Forge deterministic execution fabric.
//!
//! Responsible for generating and evaluating Nix expressions that produce
//! reproducible build environments for block execution.

#![warn(clippy::pedantic)]
#![deny(clippy::unwrap_used)]
