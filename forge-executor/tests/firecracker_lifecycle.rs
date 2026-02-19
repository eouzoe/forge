//! Integration tests for Firecracker VM lifecycle.
//!
//! These tests require KVM and the Firecracker binary.
//! Run with: `cargo test --test firecracker_lifecycle -- --ignored`

use std::path::PathBuf;
use std::time::Instant;

use forge_executor::{FirecrackerBackend, VmConfig, VmmBackend};

fn test_backend() -> FirecrackerBackend {
    FirecrackerBackend::new(
        PathBuf::from("firecracker"),
        PathBuf::from("/tmp/forge-test-sockets"),
        PathBuf::from("/tmp/forge-test-snapshots"),
    )
}

fn test_config() -> VmConfig {
    VmConfig::new(
        PathBuf::from("/home/eouzoe/src/active/forge/test-assets/vmlinux.bin"),
        PathBuf::from("/home/eouzoe/src/active/forge/test-assets/rootfs.ext4"),
    )
}

#[tokio::test]
#[ignore = "requires KVM and Firecracker binary"]
async fn spawn_vm_starts_and_responds() {
    let backend = test_backend();
    let config = test_config();

    let start = Instant::now();
    let handle = backend.spawn(&config).await.expect("VM spawn failed");
    let boot_time = start.elapsed();

    println!("VM boot time: {boot_time:?}");
    println!("VM id: {}", handle.id);
    println!("VM socket: {}", handle.socket_path.display());

    // Verify process is alive
    assert!(
        handle.socket_path.exists(),
        "socket should exist while VM is running"
    );

    backend.terminate(handle).await.expect("terminate failed");
}

#[tokio::test]
#[ignore = "requires KVM and Firecracker binary"]
async fn snapshot_creates_recoverable_state() {
    let backend = test_backend();
    let config = test_config();

    let handle = backend.spawn(&config).await.expect("VM spawn failed");
    let vm_id = handle.id;

    // Give the VM a moment to fully boot
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let snapshot_id = backend.snapshot(&handle).await.expect("snapshot failed");
    println!("Snapshot id: {snapshot_id}");

    // Verify snapshot files exist
    let mem_path = PathBuf::from(format!("/tmp/forge-test-snapshots/{snapshot_id}.mem"));
    let state_path = PathBuf::from(format!("/tmp/forge-test-snapshots/{snapshot_id}.state"));

    assert!(mem_path.exists(), "snapshot mem file should exist");
    assert!(state_path.exists(), "snapshot state file should exist");

    println!("Snapshot mem size: {} bytes", mem_path.metadata().map(|m| m.len()).unwrap_or(0));

    backend.terminate(handle).await.expect("terminate failed");
    println!("Snapshot test passed for VM {vm_id}");
}

#[tokio::test]
#[ignore = "requires KVM and Firecracker binary"]
async fn restore_from_snapshot_succeeds() {
    let backend = test_backend();
    let config = test_config();

    // Spawn original VM
    let handle = backend.spawn(&config).await.expect("VM spawn failed");
    let original_id = handle.id;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Snapshot
    let snapshot_id = backend.snapshot(&handle).await.expect("snapshot failed");

    // Terminate original
    backend.terminate(handle).await.expect("terminate original failed");
    println!("Original VM {original_id} terminated");

    // Restore from snapshot
    let start = Instant::now();
    let restored = backend.restore(&snapshot_id).await.expect("restore failed");
    let restore_time = start.elapsed();

    println!("Restore time: {restore_time:?}");
    println!("Restored VM id: {}", restored.id);

    assert!(
        restored.socket_path.exists(),
        "restored VM socket should exist"
    );
    assert_ne!(restored.id, original_id, "restored VM should have a new ID");

    backend.terminate(restored).await.expect("terminate restored failed");
}
