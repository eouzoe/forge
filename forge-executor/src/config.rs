//! VM configuration and snapshot identifier types.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Configuration for spawning a new microVM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VmConfig {
    /// Path to the Linux kernel image (vmlinux or bzImage).
    pub kernel_path: PathBuf,

    /// Path to the root filesystem image (ext4).
    pub rootfs_path: PathBuf,

    /// Number of virtual CPUs to allocate.
    pub vcpu_count: u8,

    /// Memory size in mebibytes.
    pub mem_size_mib: u32,

    /// Kernel boot arguments.
    pub boot_args: String,
}

impl VmConfig {
    /// Create a minimal VM config with sensible defaults.
    ///
    /// # Arguments
    /// - `kernel_path`: path to the kernel image
    /// - `rootfs_path`: path to the root filesystem
    #[must_use]
    pub fn new(kernel_path: PathBuf, rootfs_path: PathBuf) -> Self {
        Self {
            kernel_path,
            rootfs_path,
            vcpu_count: 1,
            mem_size_mib: 128,
            boot_args: "console=ttyS0 reboot=k panic=1 pci=off".to_owned(),
        }
    }
}

/// Opaque identifier for a VM snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SnapshotId(pub Uuid);

impl SnapshotId {
    /// Create a new random snapshot ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SnapshotId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SnapshotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
