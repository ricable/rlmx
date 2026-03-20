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
    if [ -z "$INPUT" ] || ! echo "$INPUT" | jq -e 'type == "object"' &>/dev/null; then
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
        monthly) ANNUAL_IMPACT=$(awk "BEGIN {printf \"%.2f\", $AMOUNT * 12}") ;;
        quarterly) ANNUAL_IMPACT=$(awk "BEGIN {printf \"%.2f\", $AMOUNT * 4}") ;;
        yearly) ANNUAL_IMPACT="$AMOUNT" ;;
    esac
fi

# Log savings event
LOG_DIR="${RLMX_LOG_DIR:-${XDG_STATE_HOME:-$HOME/.local/state}/rlmx/savings}"
mkdir -p "$LOG_DIR"
jq -n --arg ts "$TIMESTAMP" --arg agent "$AGENT" --argjson amount "$AMOUNT" \
  --argjson recurring "$RECURRING" --arg freq "$FREQUENCY" --arg provider "$PROVIDER" \
  --argjson annual "$ANNUAL_IMPACT" \
  '{timestamp: $ts, agent: $agent, amount: $amount, recurring: $recurring, frequency: $freq, provider: $provider, annual_impact: $annual}' \
  >> "${LOG_DIR}/savings.jsonl"

# Guard: skip notification for zero-amount savings
if awk "BEGIN {exit !($AMOUNT == 0)}"; then
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

jq -n \
  --arg notify_title "$NOTIFY_TITLE" \
  --arg notify_body "$NOTIFY_BODY" \
  --arg timestamp "$TIMESTAMP" \
  --arg agent "$AGENT" \
  --arg action "$ACTION" \
  --argjson amount "$AMOUNT" \
  --argjson recurring "$RECURRING" \
  --arg frequency "$FREQUENCY" \
  --argjson annual_impact "$ANNUAL_IMPACT" \
  --arg provider "$PROVIDER" \
  --arg category "$CATEGORY" \
  '{
    notification: {
      title: $notify_title,
      body: $notify_body,
      priority: "high",
      channel: "savings"
    },
    savings_event: {
      timestamp: $timestamp,
      agent: $agent,
      action: $action,
      amount: $amount,
      recurring: $recurring,
      frequency: $frequency,
      annual_impact: $annual_impact,
      provider: $provider,
      category: $category
    },
    engagement_points: 25,
    achievement_check: "savings_milestone"
  }'
