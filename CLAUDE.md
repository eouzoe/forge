# CLAUDE.md — Forge (Source of Truth)

> 每個 Agent 的第一件事就是讀這個文件。

## 0. 專案概述

**Forge** — 確定性執行 Fabric。Nix 可重現性 + Firecracker microVM 隔離 + SHA-256 輸出驗證。

- 語言: Rust (stable 1.92+)
- 用戶語言: 繁體中文
- 品質標準: clippy pedantic 零警告
- GitHub: github.com/eouzoe/forge

## 1. Workspace 結構

```
forge/
├── forge-core/       — Block, ExecutionRecord, TrustScore, ContentHash
├── forge-executor/   — VmmBackend trait, FirecrackerBackend, BlockRunner
├── forge-nix/        — (空) Nix derivation 管理
├── forge-auditor/    — (空) 審計引擎
└── forge-gateway/    — (建設中) Axum HTTP server for SandboX 整合
```

## 2. 核心類型

### forge-core
- `Block` — 自包含能力單元 (manifest + trust_score)
- `BlockManifest` — name, version, description, author
- `ExecutionRecord` — block_id, executor, input_hash, output_hash, duration, status
- `TrustScore` — 0.0..=1.0 信任分數
- `ContentHash` — SHA-256 包裝，Display 為 hex

### forge-executor
- `VmmBackend` trait — spawn, snapshot, restore, terminate, execute_command
- `FirecrackerBackend` — Firecracker 實作，Unix socket API
- `BlockRunner` — 執行 Block，計算 output_hash
- `ExecutionOutput` — { stdout, stderr, exit_code } (base64 分離)
- `ExecutorError` — thiserror enum, #[non_exhaustive]

## 3. 關鍵設計決策

- stdout/stderr 分離：base64 編碼 + 協議標記 (FORGE_STDOUT_B64_START/END)
- init script 嵌入 kernel boot args，VM 執行完自動 poweroff
- VmmBackend 抽象：今天 Firecracker，未來 libkrun
- 錯誤處理：thiserror everywhere, #![deny(clippy::unwrap_used)]

## 4. 當前目標：SandboX 整合

Deepractice 的 SandboX (github.com/Deepractice/SandboX) 是多隔離策略沙箱。
IsolatorType 已有 "none" | "srt" | "cloudflare" | "e2b"，e2b 未實作。

forge-gateway 提供 HTTP API，讓 TypeScript ForgeIsolator 調用：
- POST /v1/sandbox → 建立 sandbox
- POST /v1/sandbox/:id/shell → 執行命令，返回 {stdout, stderr, exit_code}
- DELETE /v1/sandbox/:id → 銷毀

## 5. 開發規範

- 格式化: `cargo fmt`
- Linting: `cargo clippy --workspace -- -D warnings -W clippy::pedantic`
- 測試: `cargo test --workspace`
- 提交: Conventional Commits (feat|fix|docs|style|refactor|perf|test|chore)
- 全局風格: `~/.claude/rules/bose-style.md` (20 章節)

## 6. 環境

| 項目 | 值 |
|------|-----|
| OS | Linux 6.6 (WSL2) |
| Rust | 1.92.0 (stable) |
| KVM | 需要 nestedVirtualization=true |
| Firecracker | 需要 /dev/kvm + kvm group |

---
**版本**: 1.0 | **最後更新**: 2026-02-21
