# Forge — 分層品質 Gate + Dashboard
#
# gate 層級（依時間成本升序）:
#   loop-gate   < 5s   — 編輯循環必跑
#   commit-gate < 30s  — 每次 commit 前必跑
#   pr-gate     < 5min — PR 前必跑
#   weekly-gate < 60min — 每周深度檢查

# ── 預設：顯示 help ────────────────────────────────────────────────────────────
default:
    @just --list --unsorted

# ── Tier 1: Loop Gate (<5s) ────────────────────────────────────────────────────
# Fast feedback: format + lint only.
loop-gate:
    cargo fmt --check
    cargo clippy --workspace -- -D warnings

# ── Tier 2: Commit Gate (<30s) ────────────────────────────────────────────────
# Everything in loop-gate + tests + supply-chain check.
commit-gate: loop-gate
    cargo test --workspace
    # TODO(Q004): create deny.toml then remove this guard
    @if [ -f deny.toml ]; then \
        cargo deny check; \
    else \
        echo "[commit-gate] SKIP: deny.toml not found — run 'just deny-init' (Q004)"; \
    fi

# ── Tier 3: PR Gate (<5min) ───────────────────────────────────────────────────
# Everything in commit-gate + coverage + docs.
pr-gate: commit-gate
    # TODO: install cargo-llvm-cov (cargo install cargo-llvm-cov)
    @if command -v cargo-llvm-cov >/dev/null 2>&1; then \
        cargo llvm-cov --workspace; \
    else \
        echo "[pr-gate] SKIP: cargo-llvm-cov not installed"; \
    fi
    cargo doc --workspace --no-deps

# ── Tier 4: Weekly Gate (<60min) ──────────────────────────────────────────────
# Everything in pr-gate + mutation testing + MIRI.
weekly-gate: pr-gate
    # TODO: install cargo-mutants (cargo install cargo-mutants)
    @if command -v cargo-mutants >/dev/null 2>&1; then \
        cargo mutants -p forge-core; \
    else \
        echo "[weekly-gate] SKIP: cargo-mutants not installed"; \
    fi
    cargo +nightly miri test --workspace --exclude forge-gateway -- --test-threads=1

# ── Outer-to-Inner Gate (<5min) ───────────────────────────────────────────────
# ADR-002: code from research/controlled circles must pass before merging to main.
# Purity: inner circle [GUARANTEED]. Label: all checks must pass, no advisory skips.
outer-to-inner-gate:
    cargo build --release
    @if [ -f deny.toml ]; then \
        cargo deny check; \
    else \
        echo "[outer-to-inner-gate] SKIP: deny.toml not found — run 'just deny-init' (Q004)"; \
    fi
    cargo audit
    cargo clippy --workspace -- -D warnings
    cargo nextest run --workspace

# ── Dashboard ─────────────────────────────────────────────────────────────────
# Single-command status overview.
dashboard:
    @echo "════════════════════════════════════════════════"
    @echo " Forge Dashboard"
    @echo "════════════════════════════════════════════════"
    @echo ""
    @echo "── Git ──────────────────────────────────────────"
    @git status --short
    @echo ""
    @echo "── Tests ────────────────────────────────────────"
    @cargo test --workspace --quiet 2>&1 | tail -3
    @echo ""
    @echo "── Clippy ───────────────────────────────────────"
    @cargo clippy --workspace -- -D warnings 2>&1 | tail -5
    @echo ""
    @echo "── Coverage ─────────────────────────────────────"
    @if command -v cargo-llvm-cov >/dev/null 2>&1; then \
        cargo llvm-cov --workspace --summary-only 2>&1 | tail -5; \
    else \
        echo "  n/a (cargo-llvm-cov not installed)"; \
    fi
    @echo ""
    @echo "════════════════════════════════════════════════"

# ── Helpers ───────────────────────────────────────────────────────────────────

# Initialise deny.toml skeleton (Q004 prerequisite)
deny-init:
    @echo "TODO(Q004): copy deny.toml from bose-search and adjust for forge"
    @echo "  See feature_list.json Q004 for steps."
