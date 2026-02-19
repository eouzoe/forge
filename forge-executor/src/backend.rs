//! VMM backend abstraction trait.
//!
//! Allows swapping between Firecracker, libkrun, or other VMMs
//! without changing the orchestration logic.

use async_trait::async_trait;

use crate::{ExecutorError, SnapshotId, VmConfig, VmHandle};

/// Virtual Machine Manager abstraction.
///
/// Implementations must be `Send + Sync` to allow use across async tasks.
///
/// # Cancel Safety
/// All methods are cancel safe. Dropping the future at any await
/// point will not leave VM state inconsistent.
#[async_trait]
pub trait VmmBackend: Send + Sync {
    /// Spawn a new VM from the given image configuration.
    ///
    /// # Errors
    /// Returns [`ExecutorError::KvmUnavailable`] if `/dev/kvm` is not accessible.
    /// Returns [`ExecutorError::SpawnFailed`] if the process cannot be started.
    async fn spawn(&self, config: &VmConfig) -> Result<VmHandle, ExecutorError>;

    /// Create a snapshot of a running VM.
    ///
    /// # Errors
    /// Returns [`ExecutorError::SnapshotFailed`] if the snapshot API call fails.
    async fn snapshot(&self, handle: &VmHandle) -> Result<SnapshotId, ExecutorError>;

    /// Restore a VM from a snapshot.
    ///
    /// # Errors
    /// Returns [`ExecutorError::RestoreFailed`] if the snapshot file is missing or corrupt.
    async fn restore(&self, snapshot_id: &SnapshotId) -> Result<VmHandle, ExecutorError>;

    /// Terminate a running VM and clean up resources.
    ///
    /// # Errors
    /// Returns [`ExecutorError::Io`] if the process cannot be killed.
    async fn terminate(&self, handle: VmHandle) -> Result<(), ExecutorError>;

    /// Check if the backend is available and properly configured.
    ///
    /// # Errors
    /// Returns [`ExecutorError::BinaryNotFound`] or [`ExecutorError::KvmUnavailable`]
    /// if the environment is not ready.
    async fn health_check(&self) -> Result<(), ExecutorError>;
}
