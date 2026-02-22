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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vm_config_custom_vcpu_and_mem_preserved() {
        let mut config = VmConfig::new(
            PathBuf::from("/tmp/vmlinux"),
            PathBuf::from("/tmp/rootfs.ext4"),
        );
        config.vcpu_count = 4;
        config.mem_size_mib = 512;
        assert_eq!(config.vcpu_count, 4, "custom vcpu_count must be preserved");
        assert_eq!(config.mem_size_mib, 512, "custom mem_size_mib must be preserved");
    }

    #[test]
    fn vm_config_serialization_roundtrip() {
        let config = VmConfig::new(
            PathBuf::from("/tmp/vmlinux"),
            PathBuf::from("/tmp/rootfs.ext4"),
        );
        let json = match serde_json::to_string(&config) {
            Ok(s) => s,
            Err(e) => panic!("serialization failed: {e}"),
        };
        let restored: VmConfig = match serde_json::from_str(&json) {
            Ok(c) => c,
            Err(e) => panic!("deserialization failed: {e}"),
        };
        assert_eq!(config.kernel_path, restored.kernel_path);
        assert_eq!(config.rootfs_path, restored.rootfs_path);
        assert_eq!(config.vcpu_count, restored.vcpu_count);
        assert_eq!(config.mem_size_mib, restored.mem_size_mib);
    }

    #[test]
    fn snapshot_id_equality_same_uuid() {
        use uuid::Uuid;
        let uuid = Uuid::new_v4();
        let a = SnapshotId(uuid);
        let b = SnapshotId(uuid);
        assert_eq!(a, b, "SnapshotIds with the same UUID must be equal");
    }

    #[test]
    fn snapshot_id_display_is_uuid_format() {
        let id = SnapshotId::new();
        let s = id.to_string();
        assert_eq!(s.len(), 36, "UUID string must be 36 chars");
        assert_eq!(s.chars().filter(|&c| c == '-').count(), 4, "UUID must have 4 dashes");
    }
}
