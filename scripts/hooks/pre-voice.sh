#!/usr/bin/env bash
# pre-voice.sh — Pre-voice interaction hook
# Checks device capabilities before starting a voice session.
#
# Input (stdin): JSON with fields:
#   { "session_id": "...", "device": "...", "battery": N, "network": "..." }
#
# Output (stdout): JSON with fields:
#   { "allow": true|false, "reason": "...", "capabilities": {...} }

set -euo pipefail

INPUT=$(cat)

# Extract fields (portable: works with or without jq)
if command -v jq &>/dev/null; then
    if ! echo "$INPUT" | jq empty 2>/dev/null; then
        echo "{\"allow\": false, \"reason\": \"Invalid JSON input\", \"capabilities\": {\"asr\": false, \"nlu\": false, \"tts\": false}}"
        exit 0
    fi
    BATTERY=$(echo "$INPUT" | jq -r '.battery // 50')
    NETWORK=$(echo "$INPUT" | jq -r '.network // "wifi"')
    DEVICE=$(echo "$INPUT" | jq -r '.device // "unknown"')
else
    BATTERY=50
    NETWORK="wifi"
    DEVICE="unknown"
fi

# Check battery threshold
if [ "$BATTERY" -lt 10 ]; then
    echo "{\"allow\": false, \"reason\": \"Battery too low for voice processing (${BATTERY}%)\", \"capabilities\": {\"asr\": false, \"nlu\": false, \"tts\": false}}"
    exit 0
fi

# Check network for cloud fallback
CLOUD_AVAILABLE="true"
if [ "$NETWORK" = "none" ] || [ "$NETWORK" = "offline" ]; then
    CLOUD_AVAILABLE="false"
fi

# Determine ASR mode based on battery
ASR_MODE="on-device"
if [ "$BATTERY" -gt 80 ] && [ "$CLOUD_AVAILABLE" = "true" ]; then
    ASR_MODE="hybrid"
fi

echo "{\"allow\": true, \"reason\": \"Device ready\", \"capabilities\": {\"asr\": true, \"asr_mode\": \"${ASR_MODE}\", \"nlu\": true, \"tts\": true, \"cloud_fallback\": ${CLOUD_AVAILABLE}, \"battery\": ${BATTERY}, \"device\": \"${DEVICE}\"}}"
