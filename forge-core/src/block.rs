use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{BlockId, ContributorId, DerivationHash};
use crate::trust::{SemVer, TrustLevel, TrustScore};

/// A composable unit of deterministic functionality in the Forge registry.
///
/// Blocks are the fundamental building blocks of the execution fabric.
/// Each block is backed by a Nix derivation ensuring reproducibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Block {
    /// Unique identifier for this block.
    pub id: BlockId,
    /// Static description of the block's interface and requirements.
    pub manifest: BlockManifest,
    /// Blocks this block is composed from, if any.
    pub composed_of: Option<Vec<BlockId>>,
    /// Aggregated trust score based on execution history.
    pub trust_score: TrustScore,
    /// The contributor who authored this block.
    pub author: ContributorId,
    /// Nix store hash of the derivation producing this block's environment.
    pub nix_derivation: DerivationHash,
    /// When this block was first registered.
    pub created_at: DateTime<Utc>,
    /// When this block was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Static description of a block's interface, requirements, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BlockManifest {
    /// Human-readable name (e.g. `"git-env"`).
    pub name: String,
    /// Semantic version of this block.
    pub version: SemVer,
    /// Short description of what this block provides.
    pub description: String,
    /// Other blocks or system tools this block depends on.
    pub requires: Vec<Dependency>,
    /// Capabilities this block exposes to dependent blocks.
    pub provides: Vec<Capability>,
    /// Estimated cognitive overhead for users composing with this block.
    pub cognitive_load: CognitiveLoad,
    /// Minimum trust level required to use this block.
    pub minimum_trust_level: TrustLevel,
}

/// A dependency on another block or system capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Dependency {
    /// Name of the required block or tool (e.g. `"git"`, `"rustc"`).
    pub name: String,
    /// Semver range string (e.g. `">= 2.40"`, `"^1.93"`).
    pub version_req: String,
    /// Whether this dependency is needed at runtime or build time.
    pub kind: DependencyKind,
}

/// When a dependency is needed during the block lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DependencyKind {
    /// Required when executing the block.
    Runtime,
    /// Required only when building the block's Nix derivation.
    Build,
}

/// A named capability that a block exposes to its dependents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Capability {
    /// Capability identifier (e.g. `"git-cli"`, `"rust-toolchain"`).
    pub name: String,
    /// Version of the capability provided.
    pub version: SemVer,
}

/// Estimated cognitive overhead for users composing with a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CognitiveLoad {
    /// Minimal configuration, safe defaults, no surprises.
    Low,
    /// Some configuration required; user should read the manifest.
    Medium,
    /// Complex composition; requires understanding of internals.
    High,
}
