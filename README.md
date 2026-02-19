# Forge

There is a particular kind of frustration that every engineer knows.

You write something. It works. You hand it to someone else, and it doesn't. You hand it back to yourself three months later, and it doesn't. The machine is the same. The code is the same. And yet the world has shifted beneath it, silently, without permission.

We have spent decades building elaborate rituals to cope with this — containers, lock files, pinned versions, reproducible builds. Each one patches a symptom. None of them address the disease.

The disease is entropy. The quiet, relentless intrusion of the outside world into systems that were meant to be closed.

Forge exists because I wanted to build something where entropy has no foothold.

---

## What This Is

Forge is a deterministic execution fabric.

It runs tasks inside Firecracker microVMs, built from Nix derivations, where every input is content-addressed and every output is cryptographically verified. The same block, given the same input, produces the same output — not as an aspiration, but as a mathematical invariant.

A single Rust binary. Self-hosted. No external services. No subscriptions. No trust required beyond what you can verify yourself.

---

## The Building Block Model

Forge organises computation into **blocks** — self-contained units of capability, each described by a manifest and backed by a Nix derivation.

A block might be a Git environment. A Rust toolchain. A search engine. A security scanner. Blocks compose: a Rust development environment depends on Git, and a CI pipeline depends on both.

Each block carries a **trust score**, accumulated through usage, audit, and community verification. New blocks begin untrusted. Trust is earned, never assumed.

```
Block: rust-dev-env
├── Provides: rustc 1.93, cargo, clippy
├── Requires: git-env (>= 2.40)
├── Trust: 0.87 (142 executions, 3 audits)
├── Cognitive Load: Medium
└── Nix: /nix/store/abc123...-rust-dev-env
```

---

## Architecture

```
crates/
├── forge-core/       Domain types: Block, Manifest, ExecutionRecord, TrustScore
├── forge-executor/   VM lifecycle: VmmBackend trait, Firecracker, BlockRunner
├── forge-nix/        Nix derivation → microVM image pipeline [planned]
├── forge-auditor/    Automated review: schema, security, determinism [planned]
└── forge-gateway/    HTTP API + SSE real-time status [planned]
```

The executor is backend-agnostic. `VmmBackend` is a trait — Firecracker today, libkrun or others tomorrow. The abstraction exists not for theoretical purity, but because the right VMM depends on the deployment context, and that context will change.

---

## Determinism Guarantee

Every execution follows the same protocol:

1. A Nix derivation defines the environment. Content-addressed. Hermetic.
2. Firecracker spawns a microVM from that derivation. Isolated. Ephemeral.
3. The block's command runs to completion. Output captured via serial console.
4. SHA-256 of stdout and stderr becomes the `output_hash`.
5. An `ExecutionRecord` is written: block, input hash, output hash, duration.

Five runs of the same block with the same input produce five identical hashes. This is not a goal. It is a test.

---

## Current State

This is v0.1.0. The foundation is laid, not the cathedral.

What works:
- Complete type system for blocks, manifests, execution records, and trust
- Firecracker VM lifecycle: spawn, snapshot, restore, terminate
- Deterministic block execution with SHA-256 output verification
- 14 unit tests, 5 integration tests, clippy pedantic with zero warnings

What doesn't exist yet:
- Nix pipeline (manual rootfs for now)
- Audit engine
- HTTP API
- Documentation beyond this file

I am building this alone, and I am building it properly. Every public type has documentation. Every error type carries context. Every lint is either satisfied or explicitly acknowledged with a reason. The code is meant to be read.

---

## Known Limitations & Open Questions

Honesty about boundaries is more useful than pretending they don't exist.

**Kernel determinism.** Nix guarantees hermetic builds, but the kernel sits beneath that guarantee. Different kernel versions or CPU microcode can produce subtly different behaviour for the same binary. The kernel is already a parameter in `VmConfig`, but it is not yet content-addressed through Nix. Until it is, determinism depends on the host environment. This is the first thing `forge-nix` will address.

**Output canonicalisation.** The current SHA-256 hash covers raw stdout and stderr. Many tools emit timestamps, PIDs, or temporary paths that vary between runs. A block running `cargo build` will produce different hashes even if the compilation is semantically identical. The audit engine will need a normalisation pipeline — stripping non-deterministic artefacts before hashing — or blocks will need to declare which output fields are deterministic. This is an unsolved design problem, not a missing feature.

**Trust score integrity.** In a single-user, local deployment, trust is straightforward. In a community setting, the current accumulation model (execution count + audit count) is vulnerable to Sybil attacks. The path forward likely involves content-addressed identity (cryptographic keys, not accounts) and a Web of Trust model where trust flows from verified sources rather than accumulating from volume. This is a v0.2.0 concern, but the type system is designed to accommodate it.

---

## Quick Start

Requirements: Rust 1.82+, KVM (`/dev/kvm`), Firecracker binary

```bash
git clone https://github.com/eouzoe/forge.git
cd forge
cargo build --workspace
cargo test --workspace
```

Integration tests require KVM:
```bash
cargo test --test firecracker_lifecycle -- --ignored
cargo test --test determinism -- --ignored
```

---

## Where This Goes

The immediate path: Nix integration, an audit engine that verifies blocks automatically, and an API that lets you submit and observe executions in real time.

Further out: confidential computing with SEV-SNP attestation, so that "this code produced this output in this environment" becomes not merely an engineering claim, but a cryptographically verifiable statement. VM forking for speculative execution. A trust model that learns from usage patterns.

Much further out: a place where anyone can assemble reliable systems from verified building blocks, where the gap between "I want to build this" and "I have built this" is bridged not by magic, but by machinery that is transparent, auditable, and honest about its limitations.

---

## Licence

Apache 2.0 or MIT, at your option.

---

If you have read this far, you are likely the sort of person who cares about the same things I care about. Reproducibility. Correctness. The quiet satisfaction of a system that does exactly what it claims to do, no more and no less.

If that resonates — a conversation, an idea, a critique, a contribution — I welcome all of it. The best code I have written has come from people who told me, plainly, where I was wrong.

If you have tokens, API keys, or simply ideas worth arguing about — this project will move faster with your help, and I will not forget it. The contribution process is not yet formalised. If you want to be involved, start a conversation.

If you have nothing to offer but curiosity, that is more than enough. Curiosity is how all of this started.
