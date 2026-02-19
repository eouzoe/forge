//! Example block instances demonstrating the schema.
//!
//! These three blocks form a dependency chain:
//! `git-env` ← `rust-dev-env` ← `bose-search`.

use chrono::Utc;

use crate::block::{Block, BlockManifest, Capability, CognitiveLoad, Dependency, DependencyKind};
use crate::id::{BlockId, ContributorId, DerivationHash};
use crate::trust::{SemVer, TrustLevel, TrustScore};

/// Returns the three canonical example blocks.
///
/// # Panics
/// Never panics — all trust scores are hard-coded valid values.
#[must_use]
pub fn example_blocks() -> Vec<Block> {
    let now = Utc::now();

    let git = Block {
        id: BlockId::new(),
        manifest: BlockManifest {
            name: "git-env".to_owned(),
            version: SemVer::new(2, 43, 0),
            description: "Provides the git CLI in a reproducible Nix environment.".to_owned(),
            requires: vec![],
            provides: vec![Capability {
                name: "git-cli".to_owned(),
                version: SemVer::new(2, 43, 0),
            }],
            cognitive_load: CognitiveLoad::Low,
            minimum_trust_level: TrustLevel::Zero,
        },
        composed_of: None,
        #[expect(clippy::unwrap_used, reason = "0.9 is a valid trust score")]
        trust_score: TrustScore::new(0.9).unwrap(),
        author: ContributorId::new("forge-team"),
        nix_derivation: DerivationHash::new("ywi5ib7yrjba3k3b26yfnbx7gappr3dg"),
        created_at: now,
        updated_at: now,
    };

    let rust_dev = Block {
        id: BlockId::new(),
        manifest: BlockManifest {
            name: "rust-dev-env".to_owned(),
            version: SemVer::new(1, 82, 0),
            description: "Provides rustc and cargo via rustup in a reproducible environment."
                .to_owned(),
            requires: vec![Dependency {
                name: "git-cli".to_owned(),
                version_req: ">= 2.40".to_owned(),
                kind: DependencyKind::Runtime,
            }],
            provides: vec![
                Capability {
                    name: "rustc".to_owned(),
                    version: SemVer::new(1, 82, 0),
                },
                Capability {
                    name: "cargo".to_owned(),
                    version: SemVer::new(1, 82, 0),
                },
            ],
            cognitive_load: CognitiveLoad::Medium,
            minimum_trust_level: TrustLevel::One,
        },
        composed_of: None,
        #[expect(clippy::unwrap_used, reason = "0.85 is a valid trust score")]
        trust_score: TrustScore::new(0.85).unwrap(),
        author: ContributorId::new("forge-team"),
        nix_derivation: DerivationHash::new("3b26yfnbx7gappr3dgywi5ib7yrjba3k"),
        created_at: now,
        updated_at: now,
    };

    let bose_search = Block {
        id: BlockId::new(),
        manifest: BlockManifest {
            name: "bose-search".to_owned(),
            version: SemVer::new(0, 1, 0),
            description: "Provides the web_search MCP tool backed by SearXNG (247 engines)."
                .to_owned(),
            requires: vec![
                Dependency {
                    name: "rustc".to_owned(),
                    version_req: ">= 1.82".to_owned(),
                    kind: DependencyKind::Build,
                },
                Dependency {
                    name: "cargo".to_owned(),
                    version_req: ">= 1.82".to_owned(),
                    kind: DependencyKind::Build,
                },
            ],
            provides: vec![Capability {
                name: "web-search-mcp".to_owned(),
                version: SemVer::new(0, 1, 0),
            }],
            cognitive_load: CognitiveLoad::High,
            minimum_trust_level: TrustLevel::Two,
        },
        composed_of: None,
        #[expect(clippy::unwrap_used, reason = "0.7 is a valid trust score")]
        trust_score: TrustScore::new(0.7).unwrap(),
        author: ContributorId::new("forge-team"),
        nix_derivation: DerivationHash::new("pr3dgywi5ib7yrjba3k3b26yfnbx7gap"),
        created_at: now,
        updated_at: now,
    };

    vec![git, rust_dev, bose_search]
}
