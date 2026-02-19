//! Firecracker VMM backend implementation.
//!
//! Manages Firecracker microVM processes via the Firecracker Management API
//! (HTTP over Unix domain socket).
//!
//! # API Reference
//! Firecracker API spec: `firecracker/src/api_server/swagger/firecracker.yaml`

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use hyper::Method;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use uuid::Uuid;

use crate::backend::{ExecutionOutput, VmmBackend};
use crate::unix_client::api_request;
use crate::{ExecutorError, SnapshotId, VmConfig, VmHandle};

/// Firecracker VMM backend.
///
/// Spawns and manages Firecracker microVM processes, communicating
/// with each via its Unix socket management API.
#[derive(Debug, Clone)]
pub struct FirecrackerBackend {
    /// Path to the `firecracker` binary.
    binary_path: PathBuf,

    /// Directory where per-VM Unix sockets are created.
    socket_dir: PathBuf,

    /// Directory where snapshot files are stored.
    snapshot_dir: PathBuf,
}

impl FirecrackerBackend {
    /// Create a new backend with the given paths.
    ///
    /// # Arguments
    /// - `binary_path`: path to the `firecracker` binary
    /// - `socket_dir`: directory for Unix socket files (must be writable)
    /// - `snapshot_dir`: directory for snapshot state files (must be writable)
    #[must_use]
    pub fn new(binary_path: PathBuf, socket_dir: PathBuf, snapshot_dir: PathBuf) -> Self {
        Self {
            binary_path,
            socket_dir,
            snapshot_dir,
        }
    }

    /// Create a backend using system defaults.
    ///
    /// Looks for `firecracker` in `$PATH`, uses `/tmp/forge-sockets` and
    /// `/tmp/forge-snapshots` for runtime directories.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(
            PathBuf::from("firecracker"),
            PathBuf::from("/tmp/forge-sockets"),
            PathBuf::from("/tmp/forge-snapshots"),
        )
    }

    fn socket_path(&self, vm_id: Uuid) -> PathBuf {
        self.socket_dir.join(format!("{vm_id}.sock"))
    }

    fn snapshot_mem_path(&self, snapshot_id: SnapshotId) -> PathBuf {
        self.snapshot_dir.join(format!("{snapshot_id}.mem"))
    }

    fn snapshot_state_path(&self, snapshot_id: SnapshotId) -> PathBuf {
        self.snapshot_dir.join(format!("{snapshot_id}.state"))
    }

    /// Wait for the Firecracker API socket to become available.
    async fn wait_for_socket(socket_path: &Path) -> Result<(), ExecutorError> {
        for _ in 0..50u8 {
            if socket_path.exists() {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err(ExecutorError::SpawnFailed(format!(
            "socket {} did not appear within 5s",
            socket_path.display()
        )))
    }

    /// Configure the VM via the Firecracker API and boot it.
    async fn configure_and_boot(
        socket_path: &Path,
        config: &VmConfig,
    ) -> Result<(), ExecutorError> {
        // Set kernel
        let kernel_body = serde_json::json!({
            "kernel_image_path": config.kernel_path,
            "boot_args": config.boot_args,
        });
        api_request(
            socket_path,
            Method::PUT,
            "/boot-source",
            Some(kernel_body.to_string()),
        )
        .await?;

        // Set rootfs
        let rootfs_body = serde_json::json!({
            "drive_id": "rootfs",
            "path_on_host": config.rootfs_path,
            "is_root_device": true,
            "is_read_only": false,
        });
        api_request(
            socket_path,
            Method::PUT,
            "/drives/rootfs",
            Some(rootfs_body.to_string()),
        )
        .await?;

        // Set machine config
        let machine_body = serde_json::json!({
            "vcpu_count": config.vcpu_count,
            "mem_size_mib": config.mem_size_mib,
        });
        api_request(
            socket_path,
            Method::PUT,
            "/machine-config",
            Some(machine_body.to_string()),
        )
        .await?;

        // Boot
        let boot_body = serde_json::json!({ "action_type": "InstanceStart" });
        api_request(
            socket_path,
            Method::PUT,
            "/actions",
            Some(boot_body.to_string()),
        )
        .await?;

        Ok(())
    }
}

#[async_trait]
impl VmmBackend for FirecrackerBackend {
    async fn spawn(&self, config: &VmConfig) -> Result<VmHandle, ExecutorError> {
        // Verify KVM is accessible
        if !Path::new("/dev/kvm").exists() {
            return Err(ExecutorError::KvmUnavailable {
                reason: "/dev/kvm not found".to_owned(),
            });
        }

        // Verify binary exists
        if !self.binary_path.exists() {
            // Try PATH lookup
            which_binary(&self.binary_path)?;
        }

        let vm_id = Uuid::new_v4();
        let socket_path = self.socket_path(vm_id);

        // Ensure socket directory exists
        tokio::fs::create_dir_all(&self.socket_dir).await?;

        tracing::info!(vm_id = %vm_id, socket = %socket_path.display(), "spawning Firecracker VM");

        let process = Command::new(&self.binary_path)
            .arg("--api-sock")
            .arg(&socket_path)
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| ExecutorError::SpawnFailed(format!("exec firecracker: {e}")))?;

        // Wait for socket
        Self::wait_for_socket(&socket_path).await?;

        // Configure and boot
        Self::configure_and_boot(&socket_path, config)
            .await
            .map_err(|e| ExecutorError::SpawnFailed(e.to_string()))?;

        tracing::info!(vm_id = %vm_id, "VM booted successfully");

        Ok(VmHandle::new(vm_id, socket_path, process))
    }

    async fn snapshot(&self, handle: &VmHandle) -> Result<SnapshotId, ExecutorError> {
        let snapshot_id = SnapshotId::new();

        tokio::fs::create_dir_all(&self.snapshot_dir).await?;

        let mem_path = self.snapshot_mem_path(snapshot_id);
        let state_path = self.snapshot_state_path(snapshot_id);

        tracing::info!(
            vm_id = %handle.id,
            snapshot_id = %snapshot_id,
            "creating VM snapshot"
        );

        // Firecracker requires the VM to be paused before snapshotting.
        let pause_body = serde_json::json!({ "state": "Paused" });
        api_request(
            &handle.socket_path,
            Method::PATCH,
            "/vm",
            Some(pause_body.to_string()),
        )
        .await
        .map_err(|e| ExecutorError::SnapshotFailed {
            vm_id: handle.id,
            reason: format!("pause failed: {e}"),
        })?;

        let body = serde_json::json!({
            "snapshot_type": "Full",
            "snapshot_path": state_path,
            "mem_file_path": mem_path,
        });

        let result = api_request(
            &handle.socket_path,
            Method::PUT,
            "/snapshot/create",
            Some(body.to_string()),
        )
        .await;

        // Always attempt to resume, even if snapshot failed.
        let resume_body = serde_json::json!({ "state": "Resumed" });
        let _ = api_request(
            &handle.socket_path,
            Method::PATCH,
            "/vm",
            Some(resume_body.to_string()),
        )
        .await;

        result.map_err(|e| ExecutorError::SnapshotFailed {
            vm_id: handle.id,
            reason: e.to_string(),
        })?;

        tracing::info!(snapshot_id = %snapshot_id, "snapshot created");

        Ok(snapshot_id)
    }

    async fn restore(&self, snapshot_id: &SnapshotId) -> Result<VmHandle, ExecutorError> {
        let mem_path = self.snapshot_mem_path(*snapshot_id);
        let state_path = self.snapshot_state_path(*snapshot_id);

        if !mem_path.exists() || !state_path.exists() {
            return Err(ExecutorError::RestoreFailed {
                snapshot_id: snapshot_id.0,
                reason: format!("snapshot files not found at {}", mem_path.display()),
            });
        }

        let vm_id = Uuid::new_v4();
        let socket_path = self.socket_path(vm_id);

        tokio::fs::create_dir_all(&self.socket_dir).await?;

        tracing::info!(
            snapshot_id = %snapshot_id,
            vm_id = %vm_id,
            "restoring VM from snapshot"
        );

        let process = Command::new(&self.binary_path)
            .arg("--api-sock")
            .arg(&socket_path)
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| ExecutorError::RestoreFailed {
                snapshot_id: snapshot_id.0,
                reason: format!("exec firecracker: {e}"),
            })?;

        Self::wait_for_socket(&socket_path)
            .await
            .map_err(|e| ExecutorError::RestoreFailed {
                snapshot_id: snapshot_id.0,
                reason: e.to_string(),
            })?;

        let body = serde_json::json!({
            "snapshot_path": state_path,
            "mem_backend": {
                "backend_path": mem_path,
                "backend_type": "File",
            },
            "enable_diff_snapshots": false,
            "resume_vm": true,
        });

        api_request(
            &socket_path,
            Method::PUT,
            "/snapshot/load",
            Some(body.to_string()),
        )
        .await
        .map_err(|e| ExecutorError::RestoreFailed {
            snapshot_id: snapshot_id.0,
            reason: e.to_string(),
        })?;

        tracing::info!(vm_id = %vm_id, "VM restored from snapshot");

        Ok(VmHandle::new(vm_id, socket_path, process))
    }

    async fn terminate(&self, mut handle: VmHandle) -> Result<(), ExecutorError> {
        tracing::info!(vm_id = %handle.id, "terminating VM");

        handle.process.kill().await?;
        let _ = tokio::fs::remove_file(&handle.socket_path).await;

        tracing::info!(vm_id = %handle.id, "VM terminated");

        Ok(())
    }

    async fn health_check(&self) -> Result<(), ExecutorError> {
        // Check KVM
        if !Path::new("/dev/kvm").exists() {
            return Err(ExecutorError::KvmUnavailable {
                reason: "/dev/kvm not found".to_owned(),
            });
        }

        // Check KVM is readable
        tokio::fs::metadata("/dev/kvm").await.map_err(|_| ExecutorError::KvmUnavailable {
            reason: "cannot access /dev/kvm (permission denied?)".to_owned(),
        })?;

        // Check binary
        which_binary(&self.binary_path)?;

        Ok(())
    }

    async fn execute_command(
        &self,
        config: &VmConfig,
        command: &str,
        timeout: Duration,
    ) -> Result<ExecutionOutput, ExecutorError> {
        // Verify KVM and binary are available.
        if !Path::new("/dev/kvm").exists() {
            return Err(ExecutorError::KvmUnavailable {
                reason: "/dev/kvm not found".to_owned(),
            });
        }
        which_binary(&self.binary_path)?;

        let vm_id = Uuid::new_v4();
        let socket_path = self.socket_path(vm_id);
        tokio::fs::create_dir_all(&self.socket_dir).await?;

        // Embed the command as the init process.
        // Markers let us extract just the command output from the serial stream.
        let init_script = format!(
            "echo FORGE_OUTPUT_START; {command} 2>&1; echo FORGE_OUTPUT_END; poweroff -f 2>/dev/null || reboot -f"
        );
        let boot_args = format!(
            "console=ttyS0 reboot=k panic=1 pci=off quiet init=/bin/sh -c \"{init_script}\""
        );

        let mut exec_config = config.clone();
        exec_config.boot_args = boot_args;

        tracing::info!(vm_id = %vm_id, %command, "executing command in microVM");

        // Spawn Firecracker with stdout piped so we can read serial console output.
        let mut process = Command::new(&self.binary_path)
            .arg("--api-sock")
            .arg(&socket_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| ExecutorError::SpawnFailed(format!("exec firecracker: {e}")))?;

        // Wait for socket, then configure and boot.
        Self::wait_for_socket(&socket_path).await?;
        Self::configure_and_boot(&socket_path, &exec_config)
            .await
            .map_err(|e| ExecutorError::SpawnFailed(e.to_string()))?;

        // Read stdout while waiting for the VM to exit (with timeout).
        let stdout_handle = process
            .stdout
            .take()
            .ok_or_else(|| ExecutorError::SpawnFailed("stdout not piped".to_owned()))?;

        let read_future = async {
            let mut buf = Vec::new();
            let mut reader = tokio::io::BufReader::new(stdout_handle);
            reader.read_to_end(&mut buf).await.map(|_| buf)
        };

        let raw_output = tokio::time::timeout(timeout, read_future)
            .await
            .map_err(|_| {
                ExecutorError::SpawnFailed(format!(
                    "VM did not complete within {}s",
                    timeout.as_secs()
                ))
            })?
            .map_err(ExecutorError::Io)?;

        // Wait for process to fully exit.
        let _ = process.wait().await;
        let _ = tokio::fs::remove_file(&socket_path).await;

        tracing::info!(vm_id = %vm_id, bytes = raw_output.len(), "VM execution complete");

        // Extract output between markers.
        let stdout = extract_marked_output(&raw_output);

        Ok(ExecutionOutput { stdout })
    }
}

/// Verify a binary exists either at the given path or in PATH.
fn which_binary(path: &Path) -> Result<(), ExecutorError> {
    if path.is_absolute() {
        if path.exists() {
            return Ok(());
        }
        return Err(ExecutorError::BinaryNotFound { path: path.to_owned() });
    }

    // Relative or bare name — check PATH
    let found = std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .map(|dir| Path::new(dir).join(path))
        .any(|p| p.exists());

    if found {
        Ok(())
    } else {
        Err(ExecutorError::BinaryNotFound { path: path.to_owned() })
    }
}

/// Extract bytes between `FORGE_OUTPUT_START` and `FORGE_OUTPUT_END` markers.
///
/// Falls back to the full raw output if markers are not found (e.g. the rootfs
/// does not support the init script pattern).
fn extract_marked_output(raw: &[u8]) -> Vec<u8> {
    let start_marker = b"FORGE_OUTPUT_START\r\n";
    let end_marker = b"FORGE_OUTPUT_END";

    let start_pos = raw
        .windows(start_marker.len())
        .position(|w| w == start_marker)
        .map(|p| p + start_marker.len());

    let end_pos = raw
        .windows(end_marker.len())
        .position(|w| w == end_marker);

    match (start_pos, end_pos) {
        (Some(s), Some(e)) if s < e => raw[s..e].to_vec(),
        _ => raw.to_vec(),
    }
}
