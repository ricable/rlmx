#!/usr/bin/env bash
# Start Serena in HTTP/SSE mode for multi-agent access
uvx --from git+https://github.com/oraios/serena serena start-mcp-server \
  --context claude-code \
  --project /Users/cedric/dev/rlmx \
  --mode query-projects \
  --transport http \
  --port 8765
