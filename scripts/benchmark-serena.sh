#!/usr/bin/env bash
# Benchmark: Serena vs No-Serena for code investigation
# Measures: tokens, tool calls, wall-clock time
#
# Usage: ./scripts/benchmark-serena.sh

set -euo pipefail

QUERY="Find where DomainEvent variants related to SyscallDispatched are emitted across all crates, trace the handler chain for each emission point, and list which agent types can trigger each handler based on the permission matrix in registry.rs"

echo "=== Benchmark: Bug Investigation Query ==="
echo "Query: $QUERY"
echo ""

# Run WITHOUT Serena — disable all MCP servers via empty config file
NO_SERENA_CONFIG=$(mktemp)
echo '{"mcpServers":{}}' > "$NO_SERENA_CONFIG"

echo "--- Run 1: WITHOUT Serena MCP ---"
START1=$(date +%s)
claude -p "$QUERY" \
  --mcp-config "$NO_SERENA_CONFIG" \
  --output-format json \
  > /tmp/bench-no-serena.json 2>/tmp/bench-no-serena.log || true
END1=$(date +%s)
rm -f "$NO_SERENA_CONFIG"

if [ -s /tmp/bench-no-serena.json ]; then
  echo "Wall time: $((END1 - START1))s"
  NO_SERENA_TOKENS=$(jq -r '(.usage.input_tokens // 0) + (.usage.cache_creation_input_tokens // 0) + (.usage.cache_read_input_tokens // 0) + (.usage.output_tokens // 0)' /tmp/bench-no-serena.json 2>/dev/null || echo "N/A")
  NO_SERENA_TURNS=$(jq -r '.num_turns // "N/A"' /tmp/bench-no-serena.json 2>/dev/null || echo "N/A")
  NO_SERENA_COST=$(jq -r '.total_cost_usd // "N/A"' /tmp/bench-no-serena.json 2>/dev/null || echo "N/A")
else
  echo "WARNING: No output produced. Check /tmp/bench-no-serena.log"
  echo "Wall time: $((END1 - START1))s (failed)"
  NO_SERENA_TOKENS="N/A"
  NO_SERENA_TURNS="N/A"
  NO_SERENA_COST="N/A"
fi

# Run WITH Serena (uses project .mcp.json)
echo ""
echo "--- Run 2: WITH Serena MCP ---"
START2=$(date +%s)
claude -p "$QUERY" \
  --output-format json \
  > /tmp/bench-with-serena.json 2>/tmp/bench-with-serena.log || true
END2=$(date +%s)

if [ -s /tmp/bench-with-serena.json ]; then
  echo "Wall time: $((END2 - START2))s"
  WITH_SERENA_TOKENS=$(jq -r '(.usage.input_tokens // 0) + (.usage.cache_creation_input_tokens // 0) + (.usage.cache_read_input_tokens // 0) + (.usage.output_tokens // 0)' /tmp/bench-with-serena.json 2>/dev/null || echo "N/A")
  WITH_SERENA_TURNS=$(jq -r '.num_turns // "N/A"' /tmp/bench-with-serena.json 2>/dev/null || echo "N/A")
  WITH_SERENA_COST=$(jq -r '.total_cost_usd // "N/A"' /tmp/bench-with-serena.json 2>/dev/null || echo "N/A")
else
  echo "WARNING: No output produced. Check /tmp/bench-with-serena.log"
  echo "Wall time: $((END2 - START2))s (failed)"
  WITH_SERENA_TOKENS="N/A"
  WITH_SERENA_TURNS="N/A"
  WITH_SERENA_COST="N/A"
fi

# Compare
echo ""
echo "=== Results ==="
printf "%-25s %-15s %-15s\n" "Metric" "No Serena" "With Serena"
printf "%-25s %-15s %-15s\n" "-------------------------" "---------------" "---------------"
printf "%-25s %-15s %-15s\n" "Wall time" "$((END1 - START1))s" "$((END2 - START2))s"
printf "%-25s %-15s %-15s\n" "Turns (tool call rounds)" "$NO_SERENA_TURNS" "$WITH_SERENA_TURNS"
printf "%-25s %-15s %-15s\n" "Total tokens" "$NO_SERENA_TOKENS" "$WITH_SERENA_TOKENS"
printf "%-25s %-15s %-15s\n" "Cost (USD)" "$NO_SERENA_COST" "$WITH_SERENA_COST"
echo ""
echo "JSON outputs: /tmp/bench-no-serena.json, /tmp/bench-with-serena.json"
echo "Logs:         /tmp/bench-no-serena.log, /tmp/bench-with-serena.log"
echo "Serena dashboard: http://localhost:24282/dashboard/"
