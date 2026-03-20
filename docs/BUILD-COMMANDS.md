# Build & Development Commands

## Standard Development

```bash
cargo build --workspace              # Build all 22 crates
cargo test --workspace               # Run all tests (1,286+)
cargo test -p rlmx-kernel            # Run tests for a single crate
cargo clippy --workspace -- -D warnings  # Lint (must be zero warnings)
cargo fmt --check                    # Check formatting
cargo fmt                            # Format code
```

## Run the MCP Server

```bash
cargo run -p rlmx-cli -- serve --port 3000      # Start MCP + WS server
```

## Voice Commands (CLI)

```bash
cargo run -p rlmx-cli -- voice start                              # Start voice pipeline
cargo run -p rlmx-cli -- voice transcribe --text "hello world"    # Simulate STT
cargo run -p rlmx-cli -- voice intents --text "I want to move to Paris"  # Show intent decomposition
cargo run -p rlmx-cli -- voice session --list                     # List voice sessions
```

## Marketplace Commands (CLI)

```bash
cargo run -p rlmx-cli -- marketplace search --domain finance      # Search agents by domain
cargo run -p rlmx-cli -- marketplace install --agent bill-negotiator  # Install agent
cargo run -p rlmx-cli -- marketplace list                         # List installed agents
cargo run -p rlmx-cli -- marketplace featured                     # Show featured agents
cargo run -p rlmx-cli -- marketplace publish --path ./agent.rvf   # Publish agent
```

## Engagement Commands (CLI)

```bash
cargo run -p rlmx-cli -- engagement score                         # Show Life Score (0-100)
cargo run -p rlmx-cli -- engagement savings                       # Show Money Saved counter
cargo run -p rlmx-cli -- engagement streak                        # Show current streak
cargo run -p rlmx-cli -- engagement achievements                  # List achievements
```

## Phone Runtime Commands (CLI)

```bash
cargo run -p rlmx-cli -- phone status                             # Phone runtime status
cargo run -p rlmx-cli -- phone agents                             # List on-device agents
cargo run -p rlmx-cli -- phone battery                            # Battery-aware scheduling info
```

## Personal Mesh Commands (CLI)

```bash
cargo run -p rlmx-cli -- mesh status                              # Personal mesh status
cargo run -p rlmx-cli -- mesh devices                             # List mesh devices
cargo run -p rlmx-cli -- mesh add-device --name pi --type hub     # Register new device
cargo run -p rlmx-cli -- mesh sync                                # Force cross-device sync
cargo run -p rlmx-cli -- mesh fleet                               # Fleet overview
cargo run -p rlmx-cli -- mesh failover                            # Failover status
```

## Billing Commands (CLI)

```bash
cargo run -p rlmx-cli -- billing status                           # Subscription status
cargo run -p rlmx-cli -- billing upgrade --tier personal          # Upgrade tier
cargo run -p rlmx-cli -- billing usage                            # Usage metrics
cargo run -p rlmx-cli -- billing family                           # Family plan info
cargo run -p rlmx-cli -- billing developer                        # Developer revenue info
```

## Federation Commands (CLI)

```bash
cargo run -p rlmx-cli -- federation status                        # Federation cycle status
cargo run -p rlmx-cli -- federation contribute                    # Trigger manual contribution
cargo run -p rlmx-cli -- federation bootstrap                     # Bootstrap from latest package
```

## Original Commands

```bash
cargo run -p rlmx-cli -- query -i "search term"    # Semantic search
cargo run -p rlmx-cli -- swarm start --zone A       # Start swarm node
cargo run -p rlmx-cli -- agent spawn --type worker   # Spawn agent
cargo run -p rlmx-cli -- research start --topic "X"  # Start research
cargo run -p rlmx-cli -- sandbox spawn --profile worker  # Spawn sandbox
```

## Edge Inference

```bash
cargo build -p rlmx-cli --features ruvllm            # CPU inference
cargo build -p rlmx-cli --features "ruvllm,metal"     # Metal GPU (macOS)
RLMX_EDGE_MODEL=tinyllama-1.1b-q4_k_m \
  cargo run -p rlmx-cli --features "ruvllm,metal" -- serve --port 3000
```

## Feature-Gated Builds (ADR-024)

```bash
cargo build -p rlmx-kernel --features ruvnet-phase1    # Phase 1: Vector & Storage
cargo build -p rlmx-swarm --features ruvnet-phase2     # Phase 2: Consensus & Networking
cargo build -p rlmx-ruvllm --features ruvnet-phase3    # Phase 3: Enhanced Inference
cargo build -p rlmx-ruvllm --features ruvnet-phase4    # Phase 4: Neural Integration
cargo build -p rlmx-kernel --features ruvnet-phase5    # Phase 5: Bare-Metal Verification
```

## Cross-Platform Bindings

```bash
# NAPI (Node.js) Bindings (ADR-020)
cd crates/rlmx-napi && cargo build --features napi     # Build NAPI module

# WASM Module (ADR-021)
cargo build -p rlmx-wasm --features wasm --target wasm32-unknown-unknown

# Full Feature Build
cargo build -p rlmx-cli --features "ruvector,ruvnet-phase1,ruvnet-phase2,ruvnet-phase3"
```

## TypeScript (@aix packages)

```bash
npm install                          # Install workspace deps
npm run build:ts                     # Build all TS packages (tsup -> CJS+ESM+.d.ts)
npm run test:ts                      # Run all 1,135 vitest tests
npm run test                         # Rust + TypeScript combined
npx tsc --noEmit -p packages/<pkg>/tsconfig.json  # Typecheck one package
npx aix --help                       # CLI entry point
npx aix serve --port 3000           # Start MCP server (TypeScript)
```

## Mobile App (React Native)

```bash
cd mobile && npm install                              # Install dependencies
cd mobile && npx react-native start                   # Start Metro bundler
JAVA_HOME=/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home \
  cd mobile/android && ./gradlew assembleDebug        # Build Android APK (requires JDK 21)
adb install mobile/android/app/build/outputs/apk/debug/app-debug.apk  # Install on device
```

## Frontend

```bash
cd frontend && python3 -m http.server 8080            # Serve web dashboard at :8080
```

## Cross-compilation (RPi5/ARM64)

```bash
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu -p rlmx-cli --features ruvllm
```

## Validation & Quality Gates

```bash
cargo test --workspace 2>&1 | grep "test result:" | awk '{sum += $4} END {print "Total:", sum}'  # Count all tests
cargo test -p rlmx-kernel -- --test-threads=1    # Single-threaded (debug race conditions)
cargo test -p rlmx-agents -- test_permission      # Run specific test pattern
RUST_LOG=debug cargo test -p rlmx-voice           # Tests with trace output
```

## Per-Crate Test Counts

| Crate | Tests | | Crate | Tests |
|-------|-------|-|-------|-------|
| rlmx-agents | 168 | | rlmx-mcp | 65 |
| rlmx-swarm | 130 | | rlmx-kernel | 64 |
| rlmx-cognitive | 74 | | rlmx-mesh | 57 |
| rlmx-voice | 69 | | rlmx-billing | 55 |
| rlmx-phone | 47 | | rlmx-marketplace | 42 |
| rlmx-federation | 42 | | rlmx-rvf | 30 |
| rlmx-napi | 29 | | rlmx-ruvllm | 24 |
| rlmx-wasm | 22 | | rlmx-cli | 11 |
| rlmx-trm | 8 | | rlmx-rlm | 5 |
| rlmx-plugin | 4 | | rlmx-artifact | 62 |
| rlmx-evolve | 87 | | rlmx-channels | 38 |
| **Total Rust** | **1,286+** |

## TypeScript Package Test Counts

| Package | Tests | | Package | Tests |
|---------|-------|-|---------|-------|
| @aix/deploy | 202 | | @aix/agents | 140 |
| @aix/marketplace | 120 | | @aix/mesh | 97 |
| @aix/plugin | 74 | | @aix/swarm | 72 |
| @aix/billing | 71 | | @aix/shared | 65 |
| @aix/federation | 50 | | @aix/mcp-server | 46 |
| @aix/rlm | 35 | | @aix/core | 17 |
| aix | 15 | | @aix/artifact | 30 |
| @aix/a2a | 51 | | @aix/skills | 48 |
| @aix/evolve | 63 | | @aix/triggers | 29 |
| @aix/channels | 23 | | @aix/billing (budget) | 18 |
| @aix/deploy (bridges) | 15 | | **Total TS** | **1,135** |

## Multi-Crate Targeted Tests

```bash
cargo test -p rlmx-voice -p rlmx-phone -p rlmx-kernel  # Voice pipeline + phone + kernel
cargo test -p rlmx-mesh -p rlmx-federation -p rlmx-billing  # Mesh ecosystem
cargo test -p rlmx-napi -p rlmx-wasm                   # Cross-platform bindings
cargo test -p rlmx-artifact -p rlmx-evolve -p rlmx-channels  # AgentOS integration crates
cargo test -p rlmx-kernel -- approval                   # Approval tier tests
cargo test -p rlmx-kernel -- trigger                    # Trigger registry tests
cargo test -p rlmx-kernel -- a2a                        # A2A protocol tests
cargo test -p rlmx-billing -- budget                    # Budget ledger tests
cargo test -p rlmx-swarm -- board                       # Coordination board tests
```

## Use Case Validation Suites

```bash
# UC1 "One Voice, Millions of Agents" (585 tests)
cargo test -p rlmx-voice -p rlmx-phone -p rlmx-marketplace -p rlmx-kernel -p rlmx-agents -p rlmx-swarm -p rlmx-mcp

# UC2 "Personal Agent Cloud" (279 tests)
cargo test -p rlmx-napi -p rlmx-wasm -p rlmx-mesh -p rlmx-federation -p rlmx-billing -p rlmx-cognitive
```

## Dependency Inspection

```bash
cargo tree -p rlmx-kernel --depth 1                 # Direct deps of a crate
cargo tree -p rlmx-kernel --depth 1 --features ruvnet-phase1  # Deps with feature enabled
cargo tree --workspace --duplicates                  # Find duplicate dependencies
```

## Release Build & Docs

```bash
cargo build --release -p rlmx-cli 2>&1 | tail -1    # Optimized binary
ls -lh target/release/rlmx-cli                      # Check binary size
cargo doc --workspace --no-deps --open               # Generate and open API docs
```

No Makefile, no CI pipeline. Default rustfmt and clippy settings apply.
