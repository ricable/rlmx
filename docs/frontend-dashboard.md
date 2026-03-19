# RLMX Frontend Dashboard

Single-page dark-themed dashboard for monitoring and controlling the RLMX distributed AI agent swarm.

## Overview

- **File**: `frontend/index.html` (~2900 lines, vanilla JS/CSS/HTML)
- **Server**: `python3 -m http.server 8080` from `frontend/`
- **URL**: `http://127.0.0.1:8080`
- **Backend**: MCP server at `:3000` (optional — works standalone with demo data)

## Design System

| Token | Value | Usage |
|-------|-------|-------|
| `--cyan` | `#00e5ff` | Primary accent, Zone A, links |
| `--violet` | `#a855f7` | Secondary accent, Zone B, agents |
| `--green` | `#10b981` | Success, healthy, Zone C |
| `--amber` | `#f59e0b` | Warning, Zone D, uptime |
| `--red` | `#ef4444` | Error, critical, degraded |
| `--pink` | `#ec4899` | Browser zone, mutations |

Fonts: JetBrains Mono (code), Outfit (display). Glass-morphism panels with blur.

## Architecture

### Layout
```
+--header (52px)----------------------------+
|  Logo  |  WS indicator  |  MCP status     |
+---+--------+------------------------------+
|   | Side   |  Content                     |
| R | bar    |  header + body (scrollable)  |
| a | (nav)  |                              |
| i |        |                              |
| l |        |                              |
+---+--------+------------------------------+
```

### Sections (12 rail icons)

1. **Swarm** — Overview, Topology Map, Health Monitor, Consensus
2. **Agents** — Agent Manager, Permission Matrix, Spawn Hierarchy
3. **Compute** — WASM Pool, Benchmark, Active Workers
4. **Memory** — Query, Ingest, Memory Stats
5. **Graph** — Graph Query (force-directed)
6. **Edge AI** — Inference, Tiered Models, Chat Format
7. **Research** — Research Tracker, Evolution View, Mutation Log
8. **Events** — Live Stream
9. **Tools** — Tool Registry
10. **JSON-RPC** — Raw Console
11. **Log** — Request History

### Key JS Modules

| Module | Type | Purpose |
|--------|------|---------|
| `DemoData` | IIFE | Generates 25 nodes, 8 agents, 3 experiments, graph, metrics |
| `SimEngine` | IIFE | 2s tick: random-walks metrics, generates events |
| `TabCoord` | IIFE | BroadcastChannel multi-tab coordination |
| `sparkline()` | Function | SVG polyline + gradient for metric cards |
| `renderForceGraph()` | Function | Fruchterman-Reingold physics on canvas |
| `drawTopology()` | Function | 5-zone topology canvas renderer |
| `viewRenderers` | Object | Maps `section.view` to `{desc, render, onShow}` |

### Data Flow

```
Page Load
  → DemoData.init()           Generate demo state
  → TabCoord.init()           Set up BroadcastChannel
  → switchSection('swarm')    Render first view
  → connect()                 Try MCP server (async)
  → connectWS()               Try WebSocket (async)
  → SimEngine.start()         Begin 2s tick loop

Every 2s (SimEngine tick):
  → DemoData.tick()           Update node metrics, generate events
  → updateSwarmMetrics()      Refresh visible metric cards + sparklines
  → renderEventStream()       Update event list if visible
  → TabCoord.broadcast()      Share state with other tabs

User clicks view:
  → switchSection() / switchView()
  → viewRenderers[key].render()    Generate HTML
  → viewRenderers[key].onShow()    Fetch data or start canvas

Action handler (e.g., doAgentList):
  → callTool('rlmx_agent_list')   Try MCP server
  → Check if result is empty       (agents.length === 0)
  → Fall back to DemoData          If empty or error
  → Render cards                   Using whichever data source
```

## Demo Data

Pre-populated on load (no server required):

| Data | Count | Details |
|------|-------|---------|
| Nodes | 25 | 7 Zone A, 8 Zone B, 5 Zone C, 3 Zone D, 2 Browser |
| Agents | 8 | Coordinator, Researcher, Router, 2 Workers, Monitor, Experimenter, Embedder |
| Experiments | 3 | Sparse attention, cross-pollination, EWC++ |
| Mutations | 8 | Generation 1-8 with fitness progression |
| Graph nodes | 12 | AuthPattern, JWTToken, UserSession, etc. |
| Graph edges | 18 | Connections between knowledge entities |
| Metrics | 60 pts | Load, memory, latency, agent count time series |
| Events | 15 | Seeded NodeJoined, HealthUpdate, AgentSpawned, etc. |
| Consensus | 3 | PBFT (view:3), Raft (term:7), Gossip (gen:42) |

## Multi-Tab / Distributed

### BroadcastChannel (Local)

Multiple tabs share state via `BroadcastChannel('rlmx-swarm')`:
- `tab-announce` — Tabs discover each other, header shows tab count
- `event` — Events propagate across tabs
- `tick` — Metrics state sync

### Remote (Planned)

Each machine runs its own MCP server. The dashboard connects to one server at a time. Future: configurable server URL in the UI, or a coordinator service that aggregates multiple servers.

## Adding a New View

1. Add nav item to the section in `sections` object
2. Add renderer to `viewRenderers`:
```javascript
'section.viewId': {
  desc: 'Description shown in content header',
  render: () => `<html string>`,
  onShow: () => { /* fetch data, start canvas */ },
},
```
3. If it needs data, add a demo data method to `DemoData`
4. If it has a canvas animation, store the RAF ID and cancel on view switch

## Backend Integration

The dashboard communicates via JSON-RPC 2.0 to `POST http://127.0.0.1:3000/mcp`:

```javascript
// Initialize (required before tools/call)
rpc('initialize', { protocolVersion: '2025-03-26', ... })

// Call a tool
callTool('rlmx_swarm_status', {})  // → { node_count, healthy_nodes, ... }
callTool('rlmx_agent_list', {})    // → { agents: [...], total }
```

WebSocket events stream from `ws://127.0.0.1:3001` with typed `SwarmEvent` payloads.
