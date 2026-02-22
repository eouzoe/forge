# Forge Threat Model

> STRIDE analysis for the Forge deterministic execution platform.
> Last updated: 2026-02-22 | Status: Phase 1 baseline

---

## 1. System Overview

Forge executes untrusted code inside Firecracker microVMs and returns
SHA-256-verified output. The attack surface spans three trust boundaries:

```
┌─────────────────────────────────────────────────────────────┐
│  Caller (SandboX / TypeScript ForgeIsolator)                │
│  Trust: UNTRUSTED — all input treated as adversarial        │
└──────────────────────┬──────────────────────────────────────┘
                       │ HTTP (POST /v1/sandbox, /shell, DELETE)
┌──────────────────────▼──────────────────────────────────────┐
│  forge-gateway  (Axum HTTP server)                          │
│  Trust: SEMI-TRUSTED — validates input, enforces limits     │
└──────────────────────┬──────────────────────────────────────┘
                       │ Rust API (VmmBackend trait)
┌──────────────────────▼──────────────────────────────────────┐
│  forge-executor  (BlockRunner + FirecrackerBackend)         │
│  Trust: TRUSTED — runs as privileged process with KVM       │
└──────────────────────┬──────────────────────────────────────┘
                       │ Unix socket (Firecracker API)
┌──────────────────────▼──────────────────────────────────────┐
│  Firecracker microVM  (guest kernel + init + user code)     │
│  Trust: UNTRUSTED — treat as fully compromised              │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Assets

| Asset | Sensitivity | Impact if Compromised |
|-------|-------------|----------------------|
| Host kernel / KVM | Critical | Full host takeover |
| forge-executor process | High | Arbitrary VM spawning, resource exhaustion |
| forge-gateway process | High | Unauthorized execution, DoS |
| Execution output / SHA-256 hash | Medium | Integrity violation, false trust scores |
| Sandbox ID (UUID) | Low | Unauthorized shell access to existing sandbox |

---

## 3. STRIDE Analysis

### 3.1 Spoofing

| Threat | Component | Mitigation | Status |
|--------|-----------|------------|--------|
| Caller forges sandbox ID to hijack another session | forge-gateway | UUID v4 (128-bit random); no sequential IDs | ✅ Implemented |
| Guest VM spoofs host via virtio channel | Firecracker | Firecracker jailer + seccomp-BPF | ⚠️ Phase 3 |
| Replay attack on `/shell` endpoint | forge-gateway | Stateless per-request; no session tokens yet | ⚠️ Phase 2 (add request signing) |

### 3.2 Tampering

| Threat | Component | Mitigation | Status |
|--------|-----------|------------|--------|
| Attacker modifies execution output in transit | forge-executor | SHA-256 `output_hash` in `ExecutionRecord` | ✅ Implemented |
| Guest writes to host filesystem via escape | Firecracker | VM disk is ephemeral; no host mounts | ✅ By design |
| Dependency supply chain compromise | All crates | `cargo audit` + `cargo deny` in CI | ⚠️ Phase 2 |

### 3.3 Repudiation

| Threat | Component | Mitigation | Status |
|--------|-----------|------------|--------|
| Caller denies submitting malicious code | forge-gateway | `ExecutionRecord` with `input_hash` + `block_id` | ✅ Implemented |
| Executor denies producing output | forge-executor | `output_hash` + `duration` in record | ✅ Implemented |
| No audit log for sandbox lifecycle | forge-gateway | Structured tracing (create/destroy events) | ⚠️ Phase 2 (persist to append-only log) |

### 3.4 Information Disclosure

| Threat | Component | Mitigation | Status |
|--------|-----------|------------|--------|
| Error messages leak host paths or secrets | forge-gateway | `GatewayError` messages are generic; no paths | ✅ Implemented |
| Guest reads host memory via speculative execution | Firecracker | KVM + Firecracker mitigations (Spectre/Meltdown) | ✅ Firecracker handles |
| Sandbox output leaks to wrong caller | forge-gateway | Response scoped to request; no shared state | ✅ By design |

### 3.5 Denial of Service

| Threat | Component | Mitigation | Status |
|--------|-----------|------------|--------|
| Caller spawns unbounded sandboxes | forge-gateway | `SandboxPool` has no limit yet | ❌ Phase 1 (add max_sandboxes config) |
| Guest runs infinite loop, exhausts CPU | Firecracker | VM CPU throttling via cgroups | ⚠️ Phase 2 (enforce vcpu budget) |
| Guest allocates all memory | Firecracker | VM memory capped at boot (mem_size_mib) | ✅ By design |
| Large command payload causes OOM in gateway | forge-gateway | No body size limit yet | ❌ Phase 1 (add `DefaultBodyLimit`) |

### 3.6 Elevation of Privilege

| Threat | Component | Mitigation | Status |
|--------|-----------|------------|--------|
| Guest escapes VM via kernel exploit | Firecracker | Firecracker minimal device model; no virtio-net by default | ✅ By design |
| forge-executor runs as root | forge-executor | Requires `kvm` group; should run as dedicated user | ⚠️ Phase 2 (systemd service with `DynamicUser`) |
| forge-gateway accepts arbitrary shell commands | forge-gateway | MVP: local exec; Phase 3: route through VM only | ⚠️ Phase 3 |

---

## 4. Known Gaps (Phase 1 Action Items)

| ID | Gap | Fix | Priority |
|----|-----|-----|----------|
| TM-001 | No `max_sandboxes` limit in `SandboxPool` | Add configurable cap, return 429 when exceeded | P1 |
| TM-002 | No HTTP body size limit | Add `axum::extract::DefaultBodyLimit` | P1 |
| TM-003 | No request signing / auth on gateway | Add bearer token or mTLS | P2 |
| TM-004 | forge-executor runs without dedicated user | systemd `DynamicUser` or dedicated `forge` user | P2 |
| TM-005 | No persistent audit log for sandbox lifecycle | Append-only JSONL log with structured events | P2 |

---

## 5. Out of Scope (v0.x)

- Multi-tenant isolation between callers (single-tenant MVP)
- Network egress from guest (disabled by default in Firecracker)
- Persistent VM snapshots (restore path not yet hardened)

---

*Update this document when trust boundaries change or new components are added.*
*Every TM-NNN gap must have a corresponding beads issue before Phase 2.*
