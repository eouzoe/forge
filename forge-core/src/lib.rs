//! Core types for the Forge deterministic execution fabric.
//!
//! Defines the fundamental domain types: blocks, execution records,
//! trust scores, and the dependency/capability model.
//!
//! See `docs/ARCHITECTURE.md` for design rationale.

#![warn(clippy::pedantic)]
#![deny(clippy::unwrap_used)]

pub mod block;
pub mod error;
pub mod examples;
pub mod execution;
pub mod id;
pub mod trust;

pub use block::{Block, BlockManifest, Capability, CognitiveLoad, Dependency, DependencyKind};
pub use error::CoreError;
pub use execution::{ExecutionRecord, ExecutionStatus};
pub use id::{BlockId, ContentHash, ContributorId, DerivationHash, ExecutionId, SnapshotId, UserId};
pub use trust::{SemVer, TrustLevel, TrustScore};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::examples::example_blocks;

    #[test]
    fn trust_score_valid_range_accepts() {
        assert!(TrustScore::new(0.0).is_ok());
        assert!(TrustScore::new(0.5).is_ok());
        assert!(TrustScore::new(1.0).is_ok());
    }

    #[test]
    fn trust_score_out_of_range_rejects() {
        assert!(TrustScore::new(-0.1).is_err());
        assert!(TrustScore::new(1.1).is_err());
        assert!(TrustScore::new(f64::NAN).is_err());
        assert!(TrustScore::new(f64::INFINITY).is_err());
    }

    #[test]
    fn content_hash_display_shows_hex() {
        let mut bytes = [0u8; 32];
        bytes[0] = 0xde;
        bytes[1] = 0xad;
        bytes[31] = 0xff;
        let hash = ContentHash::new(bytes);
        let s = hash.to_string();
        assert!(s.starts_with("dead"), "expected hex starting with 'dead', got {s}");
        assert!(s.ends_with("ff"), "expected hex ending with 'ff', got {s}");
        assert_eq!(s.len(), 64, "SHA-256 hex must be 64 chars");
    }

    #[test]
    fn example_blocks_all_valid() {
        let blocks = example_blocks();
        assert_eq!(blocks.len(), 3);

        let git = &blocks[0];
        assert_eq!(git.manifest.name, "git-env");
        assert_eq!(git.manifest.minimum_trust_level, TrustLevel::Zero);
        assert_eq!(git.manifest.cognitive_load, CognitiveLoad::Low);
        assert!(git.manifest.requires.is_empty());
        assert_eq!(git.manifest.provides.len(), 1);

        let rust_dev = &blocks[1];
        assert_eq!(rust_dev.manifest.name, "rust-dev-env");
        assert_eq!(rust_dev.manifest.minimum_trust_level, TrustLevel::One);
        assert_eq!(rust_dev.manifest.cognitive_load, CognitiveLoad::Medium);
        assert_eq!(rust_dev.manifest.requires.len(), 1);

        let bose = &blocks[2];
        assert_eq!(bose.manifest.name, "bose-search");
        assert_eq!(bose.manifest.minimum_trust_level, TrustLevel::Two);
        assert_eq!(bose.manifest.cognitive_load, CognitiveLoad::High);
        assert_eq!(bose.manifest.requires.len(), 2);
    }

    #[test]
    fn semver_display_formats_correctly() {
        let v = SemVer::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");

        let v0 = SemVer::new(0, 0, 0);
        assert_eq!(v0.to_string(), "0.0.0");
    }
}
