#!/usr/bin/env bash
# Benchmark: 5 diverse Serena queries to populate dashboard metrics
# Watch live at http://localhost:24282/dashboard/
set -euo pipefail

QUERIES=(
  "Find all implementations of the AgentType enum across Rust and TypeScript, show how each variant maps between crates/rlmx-agents and packages/shared"
  "Trace the full lifecycle of a BudgetLedger from creation through compare-and-swap updates — show all mutation points and which crates touch it"
  "List all MCP tool handlers in tools.rs, categorize them by DDD bounded context, and identify which ones are still stubs vs fully wired"
  "Find all feature gate annotations (cfg feature) across the workspace, verify each has both the feature and not-feature path as required by ADR-024"
  "Map the WebSocket event flow from SwarmEvent emission in ws.rs through to frontend consumption, showing all serialization and filtering steps"
)

LABELS=(
  "Cross-lang type mapping"
  "Budget lifecycle trace"
  "MCP tool categorization"
  "Feature gate audit"
  "WebSocket event flow"
)

OUTDIR="/tmp/bench-serena-5x"
mkdir -p "$OUTDIR"

echo "=== Serena 5-Query Benchmark ==="
echo "Dashboard: http://localhost:24282/dashboard/"
echo "Output dir: $OUTDIR"
echo ""

for i in "${!QUERIES[@]}"; do
  N=$((i + 1))
  echo "━━━ Query $N/5: ${LABELS[$i]} ━━━"
  echo "Q: ${QUERIES[$i]:0:80}..."
  echo ""

  START=$(date +%s)
  claude -p "${QUERIES[$i]}" \
    --output-format json \
    > "$OUTDIR/bench-$N.json" 2>"$OUTDIR/bench-$N.log" || true
  END=$(date +%s)

  if [ -s "$OUTDIR/bench-$N.json" ]; then
    TURNS=$(jq -r '.num_turns // "?"' "$OUTDIR/bench-$N.json")
    COST=$(jq -r '.total_cost_usd // "?"' "$OUTDIR/bench-$N.json")
    API_MS=$(jq -r '.duration_api_ms // "?"' "$OUTDIR/bench-$N.json")
    INPUT=$(jq -r '.usage.input_tokens // 0' "$OUTDIR/bench-$N.json")
    CACHE_CREATE=$(jq -r '.usage.cache_creation_input_tokens // 0' "$OUTDIR/bench-$N.json")
    CACHE_READ=$(jq -r '.usage.cache_read_input_tokens // 0' "$OUTDIR/bench-$N.json")
    OUTPUT=$(jq -r '.usage.output_tokens // 0' "$OUTDIR/bench-$N.json")
    TOTAL=$((INPUT + CACHE_CREATE + CACHE_READ + OUTPUT))

    printf "  Wall: %ss | API: %sms | Turns: %s | Tokens: %s | Cost: \$%s\n" \
      "$((END - START))" "$API_MS" "$TURNS" "$TOTAL" "$COST"
    printf "  (input: %s, cache-create: %s, cache-read: %s, output: %s)\n" \
      "$INPUT" "$CACHE_CREATE" "$CACHE_READ" "$OUTPUT"
  else
    echo "  FAILED — check $OUTDIR/bench-$N.log"
  fi
  echo ""
done

# Summary table
echo "=== Summary ==="
printf "%-4s %-28s %-8s %-8s %-10s %-10s\n" "#" "Task" "Wall(s)" "Turns" "Tokens" "Cost"
printf "%-4s %-28s %-8s %-8s %-10s %-10s\n" "---" "----------------------------" "--------" "--------" "----------" "----------"
for i in "${!LABELS[@]}"; do
  N=$((i + 1))
  if [ -s "$OUTDIR/bench-$N.json" ]; then
    WALL_END=$(jq -r '.duration_ms // 0' "$OUTDIR/bench-$N.json")
    WALL_S=$(( WALL_END / 1000 ))
    TURNS=$(jq -r '.num_turns // "?"' "$OUTDIR/bench-$N.json")
    INPUT=$(jq -r '.usage.input_tokens // 0' "$OUTDIR/bench-$N.json")
    CC=$(jq -r '.usage.cache_creation_input_tokens // 0' "$OUTDIR/bench-$N.json")
    CR=$(jq -r '.usage.cache_read_input_tokens // 0' "$OUTDIR/bench-$N.json")
    OUT=$(jq -r '.usage.output_tokens // 0' "$OUTDIR/bench-$N.json")
    TOT=$((INPUT + CC + CR + OUT))
    COST=$(jq -r '.total_cost_usd // "?"' "$OUTDIR/bench-$N.json")
    printf "%-4s %-28s %-8s %-8s %-10s \$%-9s\n" "$N" "${LABELS[$i]}" "${WALL_S}s" "$TURNS" "$TOT" "$COST"
  else
    printf "%-4s %-28s %-8s %-8s %-10s %-10s\n" "$N" "${LABELS[$i]}" "FAIL" "-" "-" "-"
  fi
done
echo ""
echo "Full results in $OUTDIR/"
echo "Serena dashboard: http://localhost:24282/dashboard/"
