#!/usr/bin/env bash
# on-agent-level.sh — Triggered when an agent levels up
# Updates agent capabilities and notifies the user.
#
# Input (stdin): JSON with fields:
#   { "agent_id": "...", "agent_name": "...", "old_level": N,
#     "new_level": N, "xp_total": N, "trigger": "..." }
#
# Output (stdout): JSON with updated capabilities and notification.

set -euo pipefail

INPUT=$(cat)

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

if command -v jq &>/dev/null; then
    if [ -z "$INPUT" ] || ! echo "$INPUT" | jq -e 'type == "object"' &>/dev/null; then
        echo "{\"error\": \"Invalid JSON input\"}"
        exit 0
    fi
    AGENT_ID=$(echo "$INPUT" | jq -r '.agent_id // "unknown"')
    AGENT_NAME=$(echo "$INPUT" | jq -r '.agent_name // "unknown"')
    OLD_LEVEL=$(echo "$INPUT" | jq -r '.old_level // 1')
    NEW_LEVEL=$(echo "$INPUT" | jq -r '.new_level // 2')
    XP_TOTAL=$(echo "$INPUT" | jq -r '.xp_total // 0')
    TRIGGER=$(echo "$INPUT" | jq -r '.trigger // "xp_threshold"')
else
    AGENT_ID="unknown"
    AGENT_NAME="unknown"
    OLD_LEVEL=1
    NEW_LEVEL=2
    XP_TOTAL=0
    TRIGGER="xp_threshold"
fi

# Determine new capabilities based on level
NEW_CAPABILITIES="[]"
MODEL_TIER="Small"
MAX_CONCURRENT=1

# Validate numeric input
[[ "$NEW_LEVEL" =~ ^[0-9]+$ ]] || NEW_LEVEL=2
[[ "$OLD_LEVEL" =~ ^[0-9]+$ ]] || OLD_LEVEL=1
[[ "$XP_TOTAL" =~ ^[0-9]+$ ]] || XP_TOTAL=0

# Progressive capability unlocks — range-based to cover all levels 1-10
if [ "$NEW_LEVEL" -ge 10 ]; then
    NEW_CAPABILITIES='["batch_processing", "proactive_scanning", "cross_agent_collab", "cloud_escalation", "autonomous_action"]'
    MODEL_TIER="Large"
    MAX_CONCURRENT=8
elif [ "$NEW_LEVEL" -ge 7 ]; then
    NEW_CAPABILITIES='["batch_processing", "proactive_scanning", "cross_agent_collab", "cloud_escalation"]'
    MODEL_TIER="Medium"
    MAX_CONCURRENT=5
elif [ "$NEW_LEVEL" -ge 5 ]; then
    NEW_CAPABILITIES='["batch_processing", "proactive_scanning", "cross_agent_collab"]'
    MODEL_TIER="Medium"
    MAX_CONCURRENT=4
elif [ "$NEW_LEVEL" -ge 3 ]; then
    NEW_CAPABILITIES='["batch_processing", "proactive_scanning"]'
    MODEL_TIER="Small"
    MAX_CONCURRENT=3
elif [ "$NEW_LEVEL" -ge 2 ]; then
    NEW_CAPABILITIES='["batch_processing"]'
    MODEL_TIER="Small"
    MAX_CONCURRENT=2
else
    NEW_CAPABILITIES='["base"]'
    MODEL_TIER="Small"
    MAX_CONCURRENT=1
fi

# Calculate XP for next level
XP_NEXT_LEVEL=$((NEW_LEVEL * NEW_LEVEL * 100))

# Log level-up event
LOG_DIR="${RLMX_LOG_DIR:-${XDG_STATE_HOME:-$HOME/.local/state}/rlmx/agents}"
mkdir -p "$LOG_DIR"
jq -n --arg ts "$TIMESTAMP" --arg aid "$AGENT_ID" --arg aname "$AGENT_NAME" \
  --argjson old "$OLD_LEVEL" --argjson new "$NEW_LEVEL" --argjson xp "$XP_TOTAL" \
  '{timestamp: $ts, agent_id: $aid, agent_name: $aname, old_level: $old, new_level: $new, xp_total: $xp}' \
  >> "${LOG_DIR}/level-ups.jsonl"

ENGAGEMENT_POINTS=$((NEW_LEVEL * 10))

jq -n \
  --arg agent_id "$AGENT_ID" \
  --arg agent_name "$AGENT_NAME" \
  --argjson old_level "$OLD_LEVEL" \
  --argjson new_level "$NEW_LEVEL" \
  --argjson xp_total "$XP_TOTAL" \
  --argjson xp_next_level "$XP_NEXT_LEVEL" \
  --arg trigger "$TRIGGER" \
  --arg timestamp "$TIMESTAMP" \
  --arg model_tier "$MODEL_TIER" \
  --argjson max_concurrent "$MAX_CONCURRENT" \
  --argjson new_capabilities "$NEW_CAPABILITIES" \
  --argjson engagement_points "$ENGAGEMENT_POINTS" \
  '{
    level_up: {
      agent_id: $agent_id,
      agent_name: $agent_name,
      old_level: $old_level,
      new_level: $new_level,
      xp_total: $xp_total,
      xp_next_level: $xp_next_level,
      trigger: $trigger,
      timestamp: $timestamp
    },
    capability_update: {
      model_tier: $model_tier,
      max_concurrent_tasks: $max_concurrent,
      new_capabilities: $new_capabilities
    },
    notification: {
      title: ($agent_name + " leveled up!"),
      body: ("Level " + ($old_level|tostring) + " -> " + ($new_level|tostring) + ". New model tier: " + $model_tier),
      priority: "medium",
      channel: "agents"
    },
    engagement_points: $engagement_points
  }'
