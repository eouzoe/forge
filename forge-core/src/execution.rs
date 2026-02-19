use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{BlockId, ContentHash, ExecutionId, SnapshotId, UserId};

/// A complete record of a single block execution.
///
/// Execution records are immutable once created and form the basis
/// for trust score computation and audit trails.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExecutionRecord {
    /// Unique identifier for this execution.
    pub id: ExecutionId,
    /// The block that was executed.
    pub block_id: BlockId,
    /// The user who triggered this execution.
    pub user_id: UserId,
    /// SHA-256 hash of the execution input.
    pub input_hash: ContentHash,
    /// SHA-256 hash of the execution output.
    pub output_hash: ContentHash,
    /// When execution began.
    pub started_at: DateTime<Utc>,
    /// Wall-clock duration of the execution.
    pub duration: Duration,
    /// VM snapshot taken after execution, if snapshotting was enabled.
    pub vm_snapshot_id: Option<SnapshotId>,
    /// Final status of the execution.
    pub status: ExecutionStatus,
}

/// The outcome of a block execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ExecutionStatus {
    /// Queued but not yet started.
    Pending,
    /// Currently running inside a microVM.
    Running,
    /// Completed successfully.
    Succeeded,
    /// Terminated with an error.
    Failed {
        /// Human-readable description of the failure.
        reason: String,
    },
}
