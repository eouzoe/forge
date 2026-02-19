//! VMM backend abstraction trait.
//!
//! Allows swapping between Firecracker, libkrun, or other VMMs
//! without changing the orchestration logic.

use std::time::Duration;

use async_trait::async_trait;

use crate::{ExecutorError, SnapshotId, VmConfig, VmHandle};

/// Raw output captured from a run-to-completion VM execution.
#[derive(Debug, Clone)]
pub struct ExecutionOutput {
    /// Bytes written to the serial console by the guest command.
    pub stdout: Vec<u8>,
}

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

    /// Spawn a VM, run `command` to completion, and return captured output.
    ///
    /// The command is embedded in the kernel boot args as the init process.
    /// The VM powers off automatically when the command exits.
    ///
    /// # Cancel Safety
    /// Cancel safe. Dropping the future will terminate the VM process.
    ///
    /// # Errors
    /// Returns [`ExecutorError::SpawnFailed`] if the VM cannot start.
    /// Returns [`ExecutorError::Io`] on timeout or process wait failure.
    async fn execute_command(
        &self,
        config: &VmConfig,
        command: &str,
        timeout: Duration,
    ) -> Result<ExecutionOutput, ExecutorError>;
}
