//! High-level VM orchestrator wrapping a [`VmmBackend`].
//!
//! Tracks active VMs and provides a safe interface for lifecycle operations.

use std::collections::HashSet;
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
    active_vms: Arc<Mutex<HashSet<Uuid>>>,
}

impl<B: VmmBackend> VmOrchestrator<B> {
    /// Create a new orchestrator backed by the given VMM.
    #[must_use]
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            active_vms: Arc::new(Mutex::new(HashSet::new())),
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
