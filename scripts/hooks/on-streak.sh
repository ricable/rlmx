#!/usr/bin/env bash
# on-streak.sh — Triggered on streak milestone
# Unlocks rewards and notifies the user when they hit streak milestones.
#
# Input (stdin): JSON with fields:
#   { "user_id": "...", "streak_days": N, "milestone": N,
#     "previous_milestone": N }
#
# Output (stdout): JSON with reward details and notification.

set -euo pipefail

INPUT=$(cat)

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

if command -v jq &>/dev/null; then
    USER_ID=$(echo "$INPUT" | jq -r '.user_id // "default"')
    STREAK_DAYS=$(echo "$INPUT" | jq -r '.streak_days // 0')
    MILESTONE=$(echo "$INPUT" | jq -r '.milestone // 0')
else
    USER_ID="default"
    STREAK_DAYS=0
    MILESTONE=0
fi

# Determine reward based on milestone
REWARD_TYPE="none"
REWARD_DESC="No reward"
BONUS_POINTS=0
UNLOCK=""

case "$MILESTONE" in
    3)
        REWARD_TYPE="boost"
        REWARD_DESC="Agent response time -10%"
        BONUS_POINTS=10
        UNLOCK="quick_start"
        ;;
    7)
        REWARD_TYPE="efficiency"
        REWARD_DESC="Agent efficiency +5%"
        BONUS_POINTS=25
        UNLOCK="week_warrior"
        ;;
    14)
        REWARD_TYPE="shield"
        REWARD_DESC="Streak Shield (miss 1 day without breaking streak)"
        BONUS_POINTS=50
        UNLOCK="streak_shield"
        ;;
    30)
        REWARD_TYPE="slot"
        REWARD_DESC="Premium Agent slot unlocked"
        BONUS_POINTS=100
        UNLOCK="month_master"
        ;;
    60)
        REWARD_TYPE="tier"
        REWARD_DESC="Priority cloud inference tier"
        BONUS_POINTS=200
        UNLOCK="dedication"
        ;;
    90)
        REWARD_TYPE="badge"
        REWARD_DESC="Gold badge + unlimited agent slots"
        BONUS_POINTS=500
        UNLOCK="gold_streak"
        ;;
    *)
        REWARD_TYPE="points"
        REWARD_DESC="Bonus engagement points"
        BONUS_POINTS=$((STREAK_DAYS * 2))
        UNLOCK=""
        ;;
esac

# Log milestone
LOG_DIR="${RLMX_LOG_DIR:-/tmp/rlmx/engagement}"
mkdir -p "$LOG_DIR"
echo "{\"timestamp\": \"${TIMESTAMP}\", \"user_id\": \"${USER_ID}\", \"streak_days\": ${STREAK_DAYS}, \"milestone\": ${MILESTONE}, \"reward\": \"${REWARD_TYPE}\"}" >> "${LOG_DIR}/streaks.jsonl"

UNLOCK_JSON="null"
if [ -n "$UNLOCK" ]; then
    UNLOCK_JSON="\"${UNLOCK}\""
fi

cat <<EOF
{
  "streak": {
    "current_days": ${STREAK_DAYS},
    "milestone_reached": ${MILESTONE},
    "timestamp": "${TIMESTAMP}"
  },
  "reward": {
    "type": "${REWARD_TYPE}",
    "description": "${REWARD_DESC}",
    "bonus_points": ${BONUS_POINTS},
    "achievement_unlock": ${UNLOCK_JSON}
  },
  "notification": {
    "title": "${STREAK_DAYS}-Day Streak!",
    "body": "${REWARD_DESC}",
    "priority": "medium",
    "channel": "engagement"
  }
}
EOF
