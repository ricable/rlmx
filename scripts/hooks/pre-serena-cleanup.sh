#!/usr/bin/env bash
# Kill stale Serena processes before starting a new session
# Prevents multiple instances from conflicting
#
# SAFETY: Skips the Serena process that Claude Code is currently using
# by checking the SERENA_PARENT_PID env var or the MCP socket owner.

set -euo pipefail

CURRENT_PPID="${SERENA_PARENT_PID:-${PPID:-0}}"

STALE_PIDS=()
while IFS= read -r pid; do
  [ -z "$pid" ] && continue
  # Skip the process owned by our current Claude Code session
  proc_ppid=$(ps -o ppid= -p "$pid" 2>/dev/null | tr -d ' ' || echo "0")
  if [ "$proc_ppid" = "$CURRENT_PPID" ]; then
    echo "[serena-cleanup] Skipping active Serena process $pid (parent=$proc_ppid)"
    continue
  fi
  STALE_PIDS+=("$pid")
done < <(pgrep -f "serena start-mcp-server" 2>/dev/null || true)

if [ "${#STALE_PIDS[@]}" -gt 0 ]; then
  for pid in "${STALE_PIDS[@]}"; do
    kill "$pid" 2>/dev/null || true
  done
  sleep 1
  echo "[serena-cleanup] Killed ${#STALE_PIDS[@]} stale Serena process(es)"
else
  echo "[serena-cleanup] No stale processes"
fi
