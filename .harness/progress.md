# Forge — Quality Hardening Progress

## 目標: 對標 Firecracker / rustc 編譯器級品質標準

核心原則：每一條品質規則都必須有對應的自動化檢查，違反時 CI 紅燈或編譯失敗。

## 現狀 (2026-02-25)

- 53 tests pass, 5 ignored, 0 failed
- lint 散落在各 crate lib.rs（無 workspace 統一）
- CI 有 test/clippy/fmt/MIRI 但 MIRI 可能壞的
- 零 unsafe 代碼，5 fuzz targets，proptest
- 無 deny.toml、無 lefthook、無 cargo-audit
- rust-version 不一致（Cargo.toml 1.82 vs CLAUDE.md 1.92+）
- release profile 無 overflow-checks

---

## Session Log

### Session 1 — 2026-02-25

**任務**: Q001 Workspace 級 lint 統一

**完成**:
- 根 Cargo.toml 加入 [workspace.lints.rust] + [workspace.lints.clippy]
- Lint groups 使用 priority = -1，個別 lint 可覆蓋
- 允許 cargo_common_metadata / multiple_crate_versions（內部 workspace）
- 所有 crate Cargo.toml 加入 [lints] workspace = true
- forge-auditor / forge-nix 改用 edition/version/license workspace = true
- 移除所有 lib.rs 手動 lint 屬性
- 補齊缺失 doc comments（pub mod、struct fields、structs）
- 修復 missing_const_for_fn、use_self、redundant_pub_crate
- cargo clippy --workspace -- -D warnings 零錯誤 ✓
- cargo test --workspace 全通過 ✓
- cargo fmt 乾淨 ✓

**Commit**: c330030

### Session 2 — 2026-02-27

**Agent**: Sonnet 4.6（環境層修復）

**工作**: forge-z1q P1 — 全域 MCP + hooks + SearXNG 自動啟動

- 全域 MCP 註冊到 ~/.claude.json：exa、context7、bose-search
- ~/.claude/settings.json 加入 PreToolUse hooks：
  - mcp__bose-search__.*：自動 `podman start bose-searxng`
  - WebFetch|WebSearch：exit 2 攔截，強制用 bose-search/Exa
- 建立 forge/.harness/restore-env.sh（一鍵恢復腳本）
- bd close forge-z1q ✓
