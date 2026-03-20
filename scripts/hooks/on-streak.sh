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
    if [ -z "$INPUT" ] || ! echo "$INPUT" | jq -e 'type == "object"' &>/dev/null; then
        echo "{\"error\": \"Invalid JSON input\"}"
        exit 0
    fi
    USER_ID=$(echo "$INPUT" | jq -r '.user_id // "default"')
    STREAK_DAYS=$(echo "$INPUT" | jq -r '.streak_days // 0')
    MILESTONE=$(echo "$INPUT" | jq -r '.milestone // 0')
else
    USER_ID="default"
    STREAK_DAYS=0
    MILESTONE=0
fi

# Validate numeric input
[[ "$STREAK_DAYS" =~ ^[0-9]+$ ]] || STREAK_DAYS=0
[[ "$MILESTONE" =~ ^[0-9]+$ ]] || MILESTONE=0

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
LOG_DIR="${RLMX_LOG_DIR:-${XDG_STATE_HOME:-$HOME/.local/state}/rlmx/engagement}"
mkdir -p "$LOG_DIR"
jq -n --arg ts "$TIMESTAMP" --arg uid "$USER_ID" \
  --argjson days "$STREAK_DAYS" --argjson ms "$MILESTONE" --arg reward "$REWARD_TYPE" \
  '{timestamp: $ts, user_id: $uid, streak_days: $days, milestone: $ms, reward: $reward}' \
  >> "${LOG_DIR}/streaks.jsonl"

UNLOCK_JSON="null"
if [ -n "$UNLOCK" ]; then
    UNLOCK_JSON="\"${UNLOCK}\""
fi

jq -n \
  --argjson streak_days "$STREAK_DAYS" \
  --argjson milestone "$MILESTONE" \
  --arg timestamp "$TIMESTAMP" \
  --arg reward_type "$REWARD_TYPE" \
  --arg reward_desc "$REWARD_DESC" \
  --argjson bonus_points "$BONUS_POINTS" \
  --argjson achievement_unlock "$UNLOCK_JSON" \
  '{
    streak: {
      current_days: $streak_days,
      milestone_reached: $milestone,
      timestamp: $timestamp
    },
    reward: {
      type: $reward_type,
      description: $reward_desc,
      bonus_points: $bonus_points,
      achievement_unlock: $achievement_unlock
    },
    notification: {
      title: (($streak_days|tostring) + "-Day Streak!"),
      body: $reward_desc,
      priority: "medium",
      channel: "engagement"
    }
  }'
