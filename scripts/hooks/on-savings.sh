#!/usr/bin/env bash
# on-savings.sh — Triggered when savings are found
# Notifies the user of money saved and updates the savings counter.
#
# Input (stdin): JSON with fields:
#   { "agent": "...", "action": "...", "amount": N,
#     "recurring": true|false, "frequency": "monthly|yearly|once",
#     "provider": "...", "category": "..." }
#
# Output (stdout): JSON with notification details and updated totals.

set -euo pipefail

INPUT=$(cat)

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

if command -v jq &>/dev/null; then
    if ! echo "$INPUT" | jq empty 2>/dev/null; then
        echo "{\"error\": \"Invalid JSON input\"}"
        exit 0
    fi
    AGENT=$(echo "$INPUT" | jq -r '.agent // "unknown"')
    ACTION=$(echo "$INPUT" | jq -r '.action // "savings found"')
    AMOUNT=$(echo "$INPUT" | jq -r '.amount // 0')
    RECURRING=$(echo "$INPUT" | jq -r '.recurring // false')
    FREQUENCY=$(echo "$INPUT" | jq -r '.frequency // "once"')
    PROVIDER=$(echo "$INPUT" | jq -r '.provider // "unknown"')
    CATEGORY=$(echo "$INPUT" | jq -r '.category // "general"')
else
    AGENT="unknown"
    ACTION="savings found"
    AMOUNT=0
    RECURRING="false"
    FREQUENCY="once"
    PROVIDER="unknown"
    CATEGORY="general"
fi

# Calculate annual impact
ANNUAL_IMPACT="$AMOUNT"
if [ "$RECURRING" = "true" ]; then
    case "$FREQUENCY" in
        monthly) ANNUAL_IMPACT=$(echo "$AMOUNT * 12" | bc 2>/dev/null || echo "$((AMOUNT * 12))") ;;
        quarterly) ANNUAL_IMPACT=$(echo "$AMOUNT * 4" | bc 2>/dev/null || echo "$((AMOUNT * 4))") ;;
        yearly) ANNUAL_IMPACT="$AMOUNT" ;;
    esac
fi

# Log savings event
LOG_DIR="${RLMX_LOG_DIR:-/tmp/rlmx/savings}"
mkdir -p "$LOG_DIR"
echo "{\"timestamp\": \"${TIMESTAMP}\", \"agent\": \"${AGENT}\", \"amount\": ${AMOUNT}, \"recurring\": ${RECURRING}, \"frequency\": \"${FREQUENCY}\", \"provider\": \"${PROVIDER}\", \"annual_impact\": ${ANNUAL_IMPACT}}" >> "${LOG_DIR}/savings.jsonl"

# Guard: skip notification for zero-amount savings
if [ "$AMOUNT" -eq 0 ] 2>/dev/null || [ "$AMOUNT" = "0" ]; then
    echo "{\"skipped\": true, \"reason\": \"Zero-amount savings ignored\"}"
    exit 0
fi

# Build notification
NOTIFY_TITLE="Money Saved!"
if [ "$RECURRING" = "true" ]; then
    NOTIFY_BODY="${AGENT} saved you \$${AMOUNT}/${FREQUENCY} with ${PROVIDER} (\$${ANNUAL_IMPACT}/year)"
else
    NOTIFY_BODY="${AGENT} saved you \$${AMOUNT} with ${PROVIDER}"
fi

cat <<EOF
{
  "notification": {
    "title": "${NOTIFY_TITLE}",
    "body": "${NOTIFY_BODY}",
    "priority": "high",
    "channel": "savings"
  },
  "savings_event": {
    "timestamp": "${TIMESTAMP}",
    "agent": "${AGENT}",
    "action": "${ACTION}",
    "amount": ${AMOUNT},
    "recurring": ${RECURRING},
    "frequency": "${FREQUENCY}",
    "annual_impact": ${ANNUAL_IMPACT},
    "provider": "${PROVIDER}",
    "category": "${CATEGORY}"
  },
  "engagement_points": 25,
  "achievement_check": "savings_milestone"
}
EOF
