//! Integration test: deterministic block execution in a microVM.
//!
//! Verifies the core MVP property: same block + same input = same output hash
//! across 5 independent VM runs.
//!
//! Requires: KVM (`/dev/kvm`) and Firecracker binary at `/usr/local/bin/firecracker`.

use std::path::PathBuf;
use std::time::Duration;

use forge_core::examples::example_blocks;
use forge_executor::{BlockRunner, FirecrackerBackend, VmConfig};

fn make_backend() -> FirecrackerBackend {
    FirecrackerBackend::new(
        PathBuf::from("/usr/local/bin/firecracker"),
        PathBuf::from("/tmp/forge-sockets"),
        PathBuf::from("/tmp/forge-snapshots"),
    )
}

fn make_vm_config() -> VmConfig {
    VmConfig::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root exists")
            .join("test-assets/vmlinux.bin"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root exists")
            .join("test-assets/rootfs.ext4"),
    )
}

/// Execute the "git-env" block 5 times and verify deterministic output.
///
/// This is the core MVP proof: same block + same input = same output hash.
#[tokio::test]
#[ignore = "requires KVM and Firecracker binary at /usr/local/bin/firecracker"]
async fn git_block_five_runs_produce_identical_hash() {
    let backend = make_backend();
    let vm_config = make_vm_config();
    let runner = BlockRunner::with_timeout(backend, vm_config, Duration::from_secs(30));

    let blocks = example_blocks();
    let git_block = &blocks[0];
    assert_eq!(git_block.manifest.name, "git-env");

    let mut hashes = Vec::with_capacity(5);
    let mut durations = Vec::with_capacity(5);

    for run in 1..=5u32 {
        let record = runner
            .execute(git_block, b"")
            .await
            .unwrap_or_else(|e| panic!("run {run} failed: {e}"));

        hashes.push(record.output_hash);
        durations.push(record.duration);

        eprintln!(
            "  Run {run}: hash={} duration={}ms",
            record.output_hash,
            record.duration.as_millis()
        );
    }

    // Print determinism report.
    let all_identical = hashes.windows(2).all(|w| w[0] == w[1]);
    eprintln!("\n=== Determinism Verification Report ===");
    eprintln!("Block: {}", git_block.manifest.name);
    eprintln!("Command: echo 'git-env'");
    eprintln!("Runs: 5");
    eprintln!("Results:");
    for (i, (hash, dur)) in hashes.iter().zip(durations.iter()).enumerate() {
        eprintln!("  Run {}: hash={hash} duration={}ms", i + 1, dur.as_millis());
    }
    eprintln!("Deterministic: {}", if all_identical { "YES (all hashes identical)" } else { "NO" });
    eprintln!("===\n");

    if !all_identical {
        panic!(
            "non-deterministic execution detected — hashes differ:\n{:#?}",
            hashes.iter().map(|h| h.to_string()).collect::<Vec<_>>()
        );
    }
}

/// Smoke test: a single VM execution completes without error.
#[tokio::test]
#[ignore = "requires KVM and Firecracker binary at /usr/local/bin/firecracker"]
async fn single_vm_execution_completes() {
    let backend = make_backend();
    let vm_config = make_vm_config();
    let runner = BlockRunner::with_timeout(backend, vm_config, Duration::from_secs(30));

    let blocks = example_blocks();
    let record = runner
        .execute(&blocks[0], b"")
        .await
        .expect("execution should succeed");

    assert_eq!(record.block_id, blocks[0].id);
    // output_hash must be non-zero (SHA-256 of non-empty output)
    assert_ne!(record.output_hash.as_bytes(), &[0u8; 32]);
}
