# Project Structure

RLMX is a hybrid Rust + TypeScript monorepo.

## Directory Layout

| Directory | Purpose |
|-----------|---------|
| `/crates` | Rust source (22 crates, each with own `src/`) |
| `/packages` | TypeScript @aix packages (20 packages) |
| `/docs` | Documentation, ADRs (39), DDD documents (16) — all `.md` files go here |
| `/frontend` | Web UI — single `index.html` (never split, keep under 3000 lines) |
| `/mobile` | React Native app (TypeScript, Android/iOS) |
| `/deploy` | Deployment configs |
| `/scripts` | Utility and hook scripts |

## Rules

- **Root folder**: NEVER save files here
- **Frontend**: single-file `frontend/index.html` — never split
- **Tests**: 1,286+ Rust tests + 1,135 TS tests (2,421+ total)
- **MCP tools**: 82 tools across the project
- **Agent types**: 17 types with permission matrix
