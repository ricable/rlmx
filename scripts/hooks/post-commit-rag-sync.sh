#!/usr/bin/env bash
# Git post-commit hook that triggers RAG reindexing
# Install: ln -sf ../../scripts/hooks/post-commit-rag-sync.sh .git/hooks/post-commit

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Find rag-sync.sh relative to this hook
if [[ -f "$SCRIPT_DIR/../../scripts/rag-sync.sh" ]]; then
    # Running from .git/hooks/
    "$SCRIPT_DIR/../../scripts/rag-sync.sh" --hook &
elif [[ -f "$SCRIPT_DIR/../rag-sync.sh" ]]; then
    # Running from scripts/hooks/
    "$SCRIPT_DIR/../rag-sync.sh" --hook &
fi
