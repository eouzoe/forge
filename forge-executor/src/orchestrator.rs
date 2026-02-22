//! High-level VM orchestrator wrapping a [`VmmBackend`].
//!
//! Tracks active VMs and provides a safe interface for lifecycle operations.

use std::collections::BTreeSet;
use std::sync::Arc;

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{ExecutorError, SnapshotId, VmConfig, VmHandle, VmmBackend};

/// High-level orchestrator for VM lifecycle management.
///
/// Wraps a [`VmmBackend`] and maintains a registry of active VMs.
/// All operations are safe to call concurrently.
pub struct VmOrchestrator<B: VmmBackend> {
    backend: B,
    active_vms: Arc<Mutex<BTreeSet<Uuid>>>,
}

impl<B: VmmBackend> VmOrchestrator<B> {
    /// Create a new orchestrator backed by the given VMM.
    #[must_use]
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            active_vms: Arc::new(Mutex::new(BTreeSet::new())),
        }
    }

    /// Spawn a new VM and register it in the active registry.
    ///
    /// # Errors
    /// Propagates errors from the underlying [`VmmBackend::spawn`].
    pub async fn spawn(&self, config: &VmConfig) -> Result<VmHandle, ExecutorError> {
        let handle = self.backend.spawn(config).await?;
        self.active_vms.lock().await.insert(handle.id);
        Ok(handle)
    }

    /// Create a snapshot of a running VM.
    ///
    /// # Errors
    /// Returns [`ExecutorError::VmNotFound`] if the VM is not registered.
    /// Propagates errors from the underlying [`VmmBackend::snapshot`].
    pub async fn snapshot(&self, handle: &VmHandle) -> Result<SnapshotId, ExecutorError> {
        if !self.active_vms.lock().await.contains(&handle.id) {
            return Err(ExecutorError::VmNotFound(handle.id));
        }
        self.backend.snapshot(handle).await
    }

    /// Restore a VM from a snapshot and register it.
    ///
    /// # Errors
    /// Propagates errors from the underlying [`VmmBackend::restore`].
    pub async fn restore(&self, snapshot_id: &SnapshotId) -> Result<VmHandle, ExecutorError> {
        let handle = self.backend.restore(snapshot_id).await?;
        self.active_vms.lock().await.insert(handle.id);
        Ok(handle)
    }

    /// Terminate a VM and remove it from the active registry.
    ///
    /// # Errors
    /// Returns [`ExecutorError::VmNotFound`] if the VM is not registered.
    /// Propagates errors from the underlying [`VmmBackend::terminate`].
    pub async fn terminate(&self, handle: VmHandle) -> Result<(), ExecutorError> {
        let vm_id = handle.id;
        if !self.active_vms.lock().await.contains(&vm_id) {
            return Err(ExecutorError::VmNotFound(vm_id));
        }
        self.backend.terminate(handle).await?;
        self.active_vms.lock().await.remove(&vm_id);
        Ok(())
    }

    /// Return the number of currently active VMs.
    pub async fn active_count(&self) -> usize {
        self.active_vms.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use async_trait::async_trait;
    use uuid::Uuid;

    use super::*;
    use crate::backend::{ExecutionOutput, VmmBackend};
    use crate::{ExecutorError, SnapshotId, VmConfig, VmHandle};

    struct AlwaysFailBackend;

    #[async_trait]
    impl VmmBackend for AlwaysFailBackend {
        async fn spawn(&self, _config: &VmConfig) -> Result<VmHandle, ExecutorError> {
            Err(ExecutorError::SpawnFailed("mock always fails".to_owned()))
        }

        async fn snapshot(&self, _handle: &VmHandle) -> Result<SnapshotId, ExecutorError> {
            Err(ExecutorError::SpawnFailed("mock".to_owned()))
        }

        async fn restore(&self, _snapshot_id: &SnapshotId) -> Result<VmHandle, ExecutorError> {
            Err(ExecutorError::SpawnFailed("mock".to_owned()))
        }

        async fn terminate(&self, _handle: VmHandle) -> Result<(), ExecutorError> {
            Ok(())
        }

        async fn health_check(&self) -> Result<(), ExecutorError> {
            Ok(())
        }

        async fn execute_command(
            &self,
            _config: &VmConfig,
            _command: &str,
            _timeout: Duration,
        ) -> Result<ExecutionOutput, ExecutorError> {
            Err(ExecutorError::SpawnFailed("mock".to_owned()))
        }
    }

    #[tokio::test]
    async fn orchestrator_active_count_starts_at_zero() {
        let orch = VmOrchestrator::new(AlwaysFailBackend);
        assert_eq!(orch.active_count().await, 0, "new orchestrator must have zero active VMs");
    }

    #[tokio::test]
    async fn orchestrator_spawn_propagates_backend_error() {
        let orch = VmOrchestrator::new(AlwaysFailBackend);
        let config = VmConfig::new(PathBuf::from("/tmp/k"), PathBuf::from("/tmp/r"));
        let result = orch.spawn(&config).await;
        assert!(
            matches!(result, Err(ExecutorError::SpawnFailed(_))),
            "spawn must propagate backend SpawnFailed"
        );
    }

    #[tokio::test]
    async fn orchestrator_terminate_unregistered_returns_vm_not_found() {
        let orch = VmOrchestrator::new(AlwaysFailBackend);
        // Spawn a real tokio child so we can build a VmHandle.
        let child = match tokio::process::Command::new("true").spawn() {
            Ok(c) => c,
            Err(e) => panic!("failed to spawn true: {e}"),
        };
        let handle = VmHandle::new(Uuid::new_v4(), PathBuf::from("/tmp/test.sock"), child);
        let result = orch.terminate(handle).await;
        assert!(
            matches!(result, Err(ExecutorError::VmNotFound(_))),
            "terminate of unregistered VM must return VmNotFound"
        );
    }

    #[tokio::test]
    async fn orchestrator_snapshot_unregistered_returns_vm_not_found() {
        let orch = VmOrchestrator::new(AlwaysFailBackend);
        let child = match tokio::process::Command::new("true").spawn() {
            Ok(c) => c,
            Err(e) => panic!("failed to spawn true: {e}"),
        };
        let handle = VmHandle::new(Uuid::new_v4(), PathBuf::from("/tmp/test.sock"), child);
        let result = orch.snapshot(&handle).await;
        assert!(
            matches!(result, Err(ExecutorError::VmNotFound(_))),
            "snapshot of unregistered VM must return VmNotFound"
        );
    }
}
