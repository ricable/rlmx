#!/usr/bin/env bash
# rag-sync.sh — CRD-inspired git-diff reconciler for qmd RAG
#
# Watches git changes and re-ingests only modified files into qmd.
# Modeled after the GitKnowledgeSource CRD + controller pattern from:
# https://devopstoolkit.live/ai/ai-coding-agents-are-blind-to-your-company-knowledge-heres-the-fix/
#
# Usage:
#   ./scripts/rag-sync.sh                    # One-shot reconcile
#   ./scripts/rag-sync.sh --watch 5m         # Continuous reconciliation every 5 minutes
#   ./scripts/rag-sync.sh --hook             # For use as git post-commit/post-merge hook
#
# The "CRD" is rag-sync.yml — declarative spec for what to watch.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SYNC_CONFIG="${PROJECT_ROOT}/scripts/rag-sync.yml"
STATE_FILE="${PROJECT_ROOT}/.rag-sync-state"
LOG_PREFIX="[rag-sync]"

# ---------------------------------------------------------------------------
# Parse the declarative spec (rag-sync.yml) — simple key extraction
# ---------------------------------------------------------------------------
get_config() {
    local key="$1"
    local default="${2:-}"
    local val
    val=$(grep "^${key}:" "$SYNC_CONFIG" 2>/dev/null | head -1 | sed 's/^[^:]*: *//' | tr -d '"' || true)
    echo "${val:-$default}"
}

# ---------------------------------------------------------------------------
# State management — track last synced commit
# ---------------------------------------------------------------------------
get_last_synced_commit() {
    if [[ -f "$STATE_FILE" ]]; then
        cat "$STATE_FILE"
    else
        echo ""
    fi
}

save_synced_commit() {
    echo "$1" > "$STATE_FILE"
}

# ---------------------------------------------------------------------------
# Change detection via git diff
# ---------------------------------------------------------------------------
detect_changes() {
    local last_commit="$1"
    local current_commit="${2:-$(git -C "$PROJECT_ROOT" rev-parse HEAD)}"

    # Git pathspecs for our file types (expanded from brace pattern)
    local -a pathspecs=( '*.md' '*.rs' '*.ts' '*.tsx' '*.toml' '*.json' )

    if [[ -z "$last_commit" ]]; then
        # First run — index everything matching patterns
        echo "$LOG_PREFIX First run, full index needed" >&2
        git -C "$PROJECT_ROOT" ls-files -- "${pathspecs[@]}" 2>/dev/null
    elif [[ "$last_commit" == "$current_commit" ]]; then
        # No changes
        echo "$LOG_PREFIX No changes since ${last_commit:0:8}" >&2
        return 0
    else
        # Diff-based — only changed files matching patterns
        echo "$LOG_PREFIX Detecting changes: ${last_commit:0:8}..${current_commit:0:8}" >&2
        git -C "$PROJECT_ROOT" diff --name-only "$last_commit" HEAD -- "${pathspecs[@]}" 2>/dev/null
    fi
}

# ---------------------------------------------------------------------------
# Reconcile — the controller loop body
# ---------------------------------------------------------------------------
reconcile() {
    local start_time
    start_time=$(date +%s)

    # Read declarative spec
    local collection
    collection=$(get_config "collection" "rlmx")
    local branch
    branch=$(get_config "branch" "main")
    local patterns
    patterns=$(get_config "patterns" "**/*.{md,rs,ts,tsx,toml,json}")

    local last_commit
    last_commit=$(get_last_synced_commit)
    local current_commit
    current_commit=$(git -C "$PROJECT_ROOT" rev-parse HEAD)

    echo "$LOG_PREFIX Reconciling collection=$collection branch=$branch"
    echo "$LOG_PREFIX Current commit: ${current_commit:0:8}"
    echo "$LOG_PREFIX Last synced:    ${last_commit:+${last_commit:0:8}}${last_commit:-(none)}"

    # Phase 1: Detect changes
    local changed_files
    changed_files=$(detect_changes "$last_commit" "$current_commit")
    local change_count
    change_count=$(echo "$changed_files" | grep -c '.' || true)

    if [[ "$change_count" -eq 0 ]]; then
        echo "$LOG_PREFIX Phase: Synced | Changes: 0 | Status: up-to-date"
        save_synced_commit "$current_commit"
        return 0
    fi

    echo "$LOG_PREFIX Phase: Ingesting | Changes: $change_count files"

    # Phase 2: Re-index via qmd (handles incremental updates internally)
    if command -v qmd &>/dev/null; then
        echo "$LOG_PREFIX Running qmd update for collection '$collection'..."
        local update_output
        update_output=$(qmd update 2>&1)
        echo "$update_output" | grep -E "(Indexed|✓|Error)" || true

        # Phase 3: Generate embeddings only if update found new content
        if echo "$update_output" | grep -qE "new|updated" && ! echo "$update_output" | grep -qE "^[^0-9]*0 new, 0 updated"; then
            echo "$LOG_PREFIX Running qmd embed for new content..."
            qmd embed 2>&1 | grep -E "(Done|✓|Error|chunks)" || true
        else
            echo "$LOG_PREFIX No new content to embed"
        fi
    else
        echo "$LOG_PREFIX WARNING: qmd not found in PATH, skipping indexing"
    fi

    # Phase 4: Update state
    save_synced_commit "$current_commit"

    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))

    # Status report (like K8s CRD status)
    echo ""
    echo "$LOG_PREFIX ┌─────────────────────────────────────────────┐"
    echo "$LOG_PREFIX │ Reconciliation Complete                     │"
    echo "$LOG_PREFIX ├─────────────┬───────────────────────────────┤"
    printf "$LOG_PREFIX │ %-11s │ %-29s │\n" "Collection" "$collection"
    printf "$LOG_PREFIX │ %-11s │ %-29s │\n" "Phase" "Synced"
    printf "$LOG_PREFIX │ %-11s │ %-29s │\n" "Changes" "$change_count files"
    printf "$LOG_PREFIX │ %-11s │ %-29s │\n" "Commit" "${current_commit:0:12}"
    printf "$LOG_PREFIX │ %-11s │ %-29s │\n" "Duration" "${duration}s"
    echo "$LOG_PREFIX └─────────────┴───────────────────────────────┘"
}

# ---------------------------------------------------------------------------
# Watch mode — continuous reconciliation (like a K8s controller)
# ---------------------------------------------------------------------------
watch_mode() {
    local interval="$1"
    # Convert interval to seconds
    local seconds
    case "$interval" in
        *m) seconds=$(( ${interval%m} * 60 )) ;;
        *s) seconds=${interval%s} ;;
        *)  seconds=$interval ;;
    esac

    echo "$LOG_PREFIX Starting watch mode (interval: ${seconds}s)"
    echo "$LOG_PREFIX Press Ctrl+C to stop"
    echo ""

    while true; do
        reconcile
        echo ""
        echo "$LOG_PREFIX Next reconciliation in ${seconds}s..."
        sleep "$seconds"
    done
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
    cd "$PROJECT_ROOT"

    # Ensure config exists
    if [[ ! -f "$SYNC_CONFIG" ]]; then
        echo "$LOG_PREFIX Creating default rag-sync.yml..."
        cat > "$SYNC_CONFIG" << 'EOF'
# rag-sync.yml — Declarative RAG sync spec (CRD-inspired)
# Modeled after GitKnowledgeSource from devopstoolkit.live
#
# This is the "Custom Resource" that defines what to watch and index.
# The controller (rag-sync.sh) reconciles actual state to desired state.

collection: rlmx
branch: main
patterns: "**/*.{md,rs,ts,tsx,toml,json}"

# Paths to include (glob patterns)
include:
  - "crates/**/*.rs"
  - "packages/**/*.ts"
  - "docs/**/*.md"
  - "*.toml"
  - "*.json"

# Paths to exclude
exclude:
  - "node_modules/**"
  - "target/**"
  - "dist/**"
  - "docs/ingested/**"

# Reconciliation settings
reconcile:
  schedule: "5m"
  on_commit: true
  on_merge: true
  full_reindex_interval: "24h"
EOF
        echo "$LOG_PREFIX Created $SYNC_CONFIG"
    fi

    case "${1:-}" in
        --watch)
            local interval="${2:-5m}"
            watch_mode "$interval"
            ;;
        --hook)
            echo "$LOG_PREFIX Triggered by git hook"
            reconcile
            ;;
        --status)
            local last
            last=$(get_last_synced_commit)
            local current
            current=$(git rev-parse HEAD)
            echo "Collection:  $(get_config collection rlmx)"
            echo "Last synced: ${last:-(never)}"
            echo "Current:     ${current:0:12}"
            if [[ "$last" == "$current" ]]; then
                echo "Status:      Synced"
            elif [[ -z "$last" ]]; then
                echo "Status:      Never synced — run rag-sync.sh to index"
            elif git cat-file -t "$last" &>/dev/null; then
                echo "Status:      Out of sync ($(git diff --name-only "$last" HEAD -- | wc -l | tr -d ' ') files changed)"
            else
                echo "Status:      Out of sync (last synced commit $last no longer valid — run --reset)"
            fi
            ;;
        --reset)
            rm -f "$STATE_FILE"
            echo "$LOG_PREFIX State reset — next run will do full index"
            ;;
        *)
            reconcile
            ;;
    esac
}

main "$@"
