#!/usr/bin/env bash
# post-voice.sh — Post-voice interaction hook
# Logs the interaction and updates engagement metrics.
#
# Input (stdin): JSON with fields:
#   { "session_id": "...", "transcript": "...", "intents": [...],
#     "duration_ms": N, "agent_used": "...", "success": true|false }
#
# Output (stdout): JSON with fields:
#   { "logged": true, "engagement_update": {...}, "streak_status": {...} }

set -euo pipefail

INPUT=$(cat)

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

if command -v jq &>/dev/null; then
    if ! echo "$INPUT" | jq empty 2>/dev/null; then
        echo "{\"logged\": false, \"error\": \"Invalid JSON input\"}"
        exit 0
    fi
    SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // "unknown"')
    DURATION=$(echo "$INPUT" | jq -r '.duration_ms // 0')
    SUCCESS=$(echo "$INPUT" | jq -r '.success // false')
    AGENT=$(echo "$INPUT" | jq -r '.agent_used // "none"')
    INTENT_COUNT=$(echo "$INPUT" | jq -r '.intents | length')
else
    SESSION_ID="unknown"
    DURATION=0
    SUCCESS="true"
    AGENT="none"
    INTENT_COUNT=0
fi

# Calculate engagement points
POINTS=5
if [ "$SUCCESS" = "true" ]; then
    POINTS=$((POINTS + 10))
fi
if [ "$INTENT_COUNT" -gt 1 ]; then
    POINTS=$((POINTS + INTENT_COUNT * 2))
fi

# Log interaction
LOG_DIR="${RLMX_LOG_DIR:-/tmp/rlmx/voice-logs}"
mkdir -p "$LOG_DIR"
echo "{\"timestamp\": \"${TIMESTAMP}\", \"session_id\": \"${SESSION_ID}\", \"duration_ms\": ${DURATION}, \"success\": ${SUCCESS}, \"agent\": \"${AGENT}\", \"points\": ${POINTS}}" >> "${LOG_DIR}/interactions.jsonl"

# Output result
cat <<EOF
{
  "logged": true,
  "log_file": "${LOG_DIR}/interactions.jsonl",
  "timestamp": "${TIMESTAMP}",
  "engagement_update": {
    "points_earned": ${POINTS},
    "interaction_count": 1,
    "streak_contribution": true
  },
  "streak_status": {
    "today_active": true,
    "consecutive_days": "check_db"
  }
}
EOF
