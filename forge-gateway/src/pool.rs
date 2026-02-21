//! In-memory sandbox lifecycle registry.
//!
//! Tracks active sandbox IDs and their metadata. In the MVP stage no VM is
//! actually spawned — the pool manages ID-to-metadata mappings only.

use std::{
    collections::HashMap,
    sync::RwLock,
    time::Instant,
};

use uuid::Uuid;

/// Metadata stored for each active sandbox.
#[derive(Debug)]
pub struct SandboxEntry {
    /// Runtime identifier, e.g. `"node"` or `"python"`.
    pub runtime: String,
    /// Wall-clock time at which the sandbox was created.
    pub created_at: Instant,
}

/// Thread-safe registry of active sandboxes.
#[derive(Debug, Default)]
pub struct SandboxPool {
    entries: RwLock<HashMap<Uuid, SandboxEntry>>,
}

impl SandboxPool {
    /// Create an empty pool.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new sandbox and return its assigned ID.
    ///
    /// # Panics
    /// Panics if the internal `RwLock` is poisoned (a previous thread panicked
    /// while holding the write lock).
    pub fn create(&self, runtime: String) -> Uuid {
        let id = Uuid::new_v4();
        #[expect(clippy::expect_used, reason = "lock poisoning is unrecoverable")]
        self.entries
            .write()
            .expect("sandbox pool write lock poisoned")
            .insert(id, SandboxEntry { runtime, created_at: Instant::now() });
        id
    }

    /// Remove a sandbox by ID. Returns `true` if it existed.
    ///
    /// # Panics
    /// Panics if the internal `RwLock` is poisoned.
    pub fn remove(&self, id: Uuid) -> bool {
        #[expect(clippy::expect_used, reason = "lock poisoning is unrecoverable")]
        self.entries
            .write()
            .expect("sandbox pool write lock poisoned")
            .remove(&id)
            .is_some()
    }

    /// Return `true` if the sandbox ID is currently registered.
    ///
    /// # Panics
    /// Panics if the internal `RwLock` is poisoned.
    #[must_use]
    pub fn contains(&self, id: Uuid) -> bool {
        #[expect(clippy::expect_used, reason = "lock poisoning is unrecoverable")]
        self.entries
            .read()
            .expect("sandbox pool read lock poisoned")
            .contains_key(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_pool_create_and_remove_lifecycle() {
        let pool = SandboxPool::new();
        let id = pool.create("node".to_owned());
        assert!(pool.contains(id), "sandbox should exist after create");
        let removed = pool.remove(id);
        assert!(removed, "remove should return true for existing sandbox");
        assert!(!pool.contains(id), "sandbox should not exist after remove");
    }

    #[test]
    fn sandbox_pool_not_found_returns_false() {
        let pool = SandboxPool::new();
        let unknown = Uuid::new_v4();
        assert!(!pool.contains(unknown), "unknown ID should not be found");
        assert!(!pool.remove(unknown), "removing unknown ID should return false");
    }
}
