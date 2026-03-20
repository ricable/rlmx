#!/usr/bin/env bash
# Start Serena in HTTP/SSE mode for multi-agent access
set -euo pipefail
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
uvx --from git+https://github.com/oraios/serena serena start-mcp-server \
  --context claude-code \
  --project "$PROJECT_ROOT" \
  --mode query-projects \
  --transport http \
  --port 8765
