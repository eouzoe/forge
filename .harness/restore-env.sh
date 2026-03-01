#!/usr/bin/env bash
# restore-env.sh — 一鍵恢復 Claude Code 開發環境
# 在 settings.json 被刪或 MCP 丟失後執行
# Usage: bash .harness/restore-env.sh

set -euo pipefail

echo "=== Forge 開發環境恢復 ==="

# 1. 全域 MCP 註冊
echo "[1/4] 註冊全域 MCP servers..."

claude mcp add exa --scope user \
  -e EXA_API_KEY="${EXA_API_KEY:?EXA_API_KEY not set}" \
  -- npx -y exa-mcp-server

claude mcp add context7 --scope user \
  -- bunx @upstash/context7-mcp@latest

claude mcp add bose-search --scope user \
  -e SEARXNG_URL="http://localhost:8080" \
  -- /home/eouzoe/src/active/bose-search/target/release/bose-mcp

echo "  ✓ exa, context7, bose-search 已註冊"

# 2. 啟動 SearXNG 容器
echo "[2/4] 啟動 SearXNG 容器..."
if podman start bose-searxng 2>/dev/null; then
  echo "  ✓ bose-searxng started"
else
  echo "  ! 容器不存在，需要手動建立（見 bose-search/README.md）"
fi

# 3. 確認 hooks 存在
echo "[3/4] 確認 hooks 設定..."
if grep -q '"PreToolUse"' ~/.claude/settings.json 2>/dev/null; then
  echo "  ✓ hooks 已設定"
else
  echo "  ! ~/.claude/settings.json 缺少 hooks，請手動加入"
  echo "    參考：forge/.harness/restore-env.sh 的 hooks 區段"
fi

# 4. 驗證
echo "[4/4] 驗證..."
claude mcp list 2>/dev/null && echo "  ✓ MCP list OK" || echo "  ! MCP list 失敗"

echo ""
echo "=== 完成 ==="
echo "下一步：cd ~/src/active/forge && claude --model sonnet --dangerously-skip-permissions"
