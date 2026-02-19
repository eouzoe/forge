//! VM handle — represents a running microVM instance.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A handle to a running Firecracker microVM.
///
/// Dropping this handle does NOT terminate the VM. Call
/// [`VmmBackend::terminate`] explicitly to clean up resources.
#[derive(Debug)]
#[non_exhaustive]
pub struct VmHandle {
    /// Unique identifier for this VM instance.
    pub id: Uuid,

    /// Path to the Firecracker API Unix socket.
    pub socket_path: PathBuf,

    /// The Firecracker child process.
    pub process: tokio::process::Child,

    /// Timestamp when the VM was created.
    pub created_at: DateTime<Utc>,
}

impl VmHandle {
    /// Create a new VM handle.
    #[must_use]
    pub fn new(id: Uuid, socket_path: PathBuf, process: tokio::process::Child) -> Self {
        Self {
            id,
            socket_path,
            process,
            created_at: Utc::now(),
        }
    }
}
