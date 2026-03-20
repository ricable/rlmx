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
LOG_DIR="${RLMX_LOG_DIR:-/tmp/rlmx/agents}"
mkdir -p "$LOG_DIR"
echo "{\"timestamp\": \"${TIMESTAMP}\", \"agent_id\": \"${AGENT_ID}\", \"agent_name\": \"${AGENT_NAME}\", \"old_level\": ${OLD_LEVEL}, \"new_level\": ${NEW_LEVEL}, \"xp_total\": ${XP_TOTAL}}" >> "${LOG_DIR}/level-ups.jsonl"

cat <<EOF
{
  "level_up": {
    "agent_id": "${AGENT_ID}",
    "agent_name": "${AGENT_NAME}",
    "old_level": ${OLD_LEVEL},
    "new_level": ${NEW_LEVEL},
    "xp_total": ${XP_TOTAL},
    "xp_next_level": ${XP_NEXT_LEVEL},
    "trigger": "${TRIGGER}",
    "timestamp": "${TIMESTAMP}"
  },
  "capability_update": {
    "model_tier": "${MODEL_TIER}",
    "max_concurrent_tasks": ${MAX_CONCURRENT},
    "new_capabilities": ${NEW_CAPABILITIES}
  },
  "notification": {
    "title": "${AGENT_NAME} leveled up!",
    "body": "Level ${OLD_LEVEL} -> ${NEW_LEVEL}. New model tier: ${MODEL_TIER}",
    "priority": "medium",
    "channel": "agents"
  },
  "engagement_points": $((NEW_LEVEL * 10))
}
EOF
