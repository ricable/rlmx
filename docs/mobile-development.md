# RuVix Mobile App -- Development Guide

## Overview

React Native 0.84.1 + TypeScript mobile app for Android. Voice-first AI agent superapp that lets users manage a collection of personal AI agents across life domains (Finance, Health, Time, Safety, and more). The app connects to the RLMX cognition kernel via MCP JSON-RPC and WebSocket, and works fully offline with rich demo data.

## Prerequisites

| Requirement       | Version              | Notes                                                             |
|--------------------|----------------------|-------------------------------------------------------------------|
| Node.js            | 22.11.0+             | See `engines` in `package.json`                                   |
| JDK                | 21                   | **NOT Java 26** -- incompatible with React Native Gradle plugin   |
| Android SDK        | API 35               | Install via Android Studio SDK Manager                            |
| Android device/emu | --                   | USB debugging enabled, or an AVD with API 35                      |
| React Native CLI   | 20.1.0               | Installed as a devDependency (`@react-native-community/cli`)      |

### JDK 21 on macOS (Homebrew)

```bash
brew install openjdk@21
export JAVA_HOME=/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home
java -version   # should print "openjdk 21.*"
```

Add the export to your shell profile (`~/.zshrc`) so it persists.

## Quick Start

```bash
cd mobile
npm install

# Build debug APK
JAVA_HOME=/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home \
  cd android && ./gradlew assembleDebug

# Install on connected device
adb install android/app/build/outputs/apk/debug/app-debug.apk
```

To run with Metro bundler (live reload):

```bash
cd mobile
npm start                     # Start Metro in one terminal
npm run android               # Build and install in another terminal
```

## Project Structure

```
mobile/
  App.tsx                      # Root component: providers + navigation
  index.js                     # React Native entry point
  package.json                 # Dependencies (RN 0.84.1, React 19.2.3)
  tsconfig.json                # TypeScript config
  babel.config.js              # Babel preset (react-native)
  metro.config.js              # Metro bundler config
  jest.config.js               # Jest test config
  app.json                     # App display name
  android/                     # Android native project (Gradle)
  ios/                         # iOS native project (unused, Android-only for now)
  __tests__/                   # Root-level tests
  src/
    screens/                   # 8 screens (see below)
    components/                # 11 reusable UI components
    hooks/                     # 3 custom React hooks
    services/                  # 6 service modules
    animations/                # 2 animation components
    data/                      # Demo agent data
    theme/                     # Colors, spacing, design tokens
    navigation/                # AppNavigator (tab + stack)
    context/                   # AppContext (global state)
    types/                     # TypeScript interfaces
```

## Screens

### 1. HomeScreen

The landing screen. Displays a personalized greeting (time-of-day aware), connection status indicator, and four key widgets:

- **LifeScoreCard** -- overall Life Score (0-100) as an animated ring
- **MoneySavedCounter** -- cumulative money saved by agents
- **StreakIndicator** -- current daily engagement streak
- **BriefingCard** -- morning briefing items with actionable suggestions per domain

### 2. VoiceScreen

Voice-first interaction hub. Central mic button with three visual states:

- **Idle** -- tap to start listening
- **Listening** -- animated pulse, speech-to-text transcript appears live
- **Processing** -- agents working, progress cards appear below

Displays real-time `AgentProgress` cards showing which domain agents are handling the parsed intents.

### 3. AgentsScreen

Grid overview of the user's collected AI agents. Each card shows name, domain, level, XP bar, and task stats. Tapping a card navigates to `AgentDetailScreen`. Includes a link to the Marketplace for discovering new agents.

### 4. InsightsScreen

Four domain insight cards (Finance, Health, Time, Safety) each showing:

- Current score (0-100)
- Sparkline trend chart (last 7 data points)
- Change percentage
- Key metrics and actionable suggestions

### 5. ProfileScreen

User profile with:

- Tier badge (Free / Plus / Pro)
- Level and XP progress bar
- Streaks calendar heatmap
- Achievement badges (12 of 50 unlocked in demo)
- Settings access

### 6. MarketplaceScreen

Agent marketplace with:

- 12 domain categories (Finance, Health, Legal, Career, Education, Home, Shopping, Travel, Social, Government, Automotive, Pet)
- Featured agent carousel
- Search and filter
- Install buttons with price tier indicators (Free / Plus / Pro)

### 7. AgentDetailScreen

Full agent profile accessed from Marketplace:

- Publisher info, version, install count
- Rating with review count
- Long description
- Required permissions list
- User reviews
- "Similar agents" section

### 8. AgentCollectionScreen

Detailed collection view with:

- 4-column grid of collected agents
- Level badges and XP bars per agent
- Sort options (by level, domain, last used)
- Locked/unlocked state visual distinction

## Components

| Component            | File                        | Description                                                      |
|----------------------|-----------------------------|------------------------------------------------------------------|
| SparklineChart       | `SparklineChart.tsx`        | Inline SVG-style sparkline for trend data (7-point arrays)       |
| LifeScoreCard        | `LifeScoreCard.tsx`         | Animated circular score gauge (0-100) with gradient ring         |
| MoneySavedCounter    | `MoneySavedCounter.tsx`     | Animated dollar counter with rolling number animation            |
| AgentCard            | `AgentCard.tsx`             | Agent tile showing icon, name, domain, level, XP bar             |
| VoiceMicButton       | `VoiceMicButton.tsx`        | Central mic button with idle/listening/processing states          |
| ProgressCard         | `ProgressCard.tsx`          | Agent task progress bar with status text                         |
| StreakIndicator       | `StreakIndicator.tsx`       | Current streak count with flame icon                             |
| BriefingCard         | `BriefingCard.tsx`          | Morning briefing item with domain icon and action button         |
| DomainCard           | `DomainCard.tsx`            | Insight card for a domain with score, sparkline, metrics         |
| AchievementBadge     | `AchievementBadge.tsx`     | Achievement icon with locked/unlocked visual state               |
| MarketplaceCard      | `MarketplaceCard.tsx`      | Marketplace agent listing card with rating, installs, price tier |

## Animations

| Module              | File                    | Description                                             |
|---------------------|-------------------------|---------------------------------------------------------|
| CounterAnimation    | `CounterAnimation.tsx`  | Smooth number-rolling animation for counters            |
| PulseAnimation      | `PulseAnimation.tsx`    | Radial pulse effect for the voice mic button            |

## Services

### api.ts

HTTP client to the RLMX MCP server at `localhost:3000`. Sends JSON-RPC 2.0 requests to `/mcp` using `tools/call` method. Exports:

- `mcpCall(method, params)` -- generic MCP tool invocation, returns `null` on failure
- `checkConnection()` -- pings `/health` endpoint, returns boolean

### voice.ts

`VoiceService` with full voice interaction pipeline. Defines 6 voice personas (`Finance`, `Health`, `Legal`, `Shopping`, `Calendar`, `Emergency`). In demo mode, simulates the entire pipeline locally:

1. Transcript capture
2. Intent decomposition (maps utterance to persona + action + entities)
3. Agent progress updates (queued -> working -> done)
4. Per-persona voice responses with confidence scores

When the server is reachable, delegates intent decomposition to the MCP endpoint.

### notifications.ts

`NotificationService` with 3-tier priority system:

- **Critical** -- always delivered (safety alerts, breaches)
- **Actionable** -- delivered unless fatigue threshold hit
- **Informational** -- suppressed when response rate drops below 30%

Fatigue prevention resets every 24 hours. Requires at least 5 notifications before fatigue logic activates.

### websocket.ts

WebSocket client connecting to `ws://localhost:3001`. Handles 12 `SwarmEvent` types mirroring `crates/rlmx-mcp/src/ws.rs`:

`NodeJoined`, `NodeLeft`, `AgentSpawned`, `AgentTerminated`, `HealthUpdate`, `ExperimentUpdate`, `MutationFound`, `SandboxSpawned`, `SandboxTerminated`, `VoiceChunk`, `AgentProgress`, `MultimodalResponse`

Includes exponential backoff reconnection (1s initial, 30s max, 10 attempts). Falls back to emitting simulated demo events when the server is unreachable.

### storage.ts

Thin wrapper around `@react-native-async-storage/async-storage`. Provides typed `save<T>`, `load<T>`, and `remove` functions. Stores user profile, agents, life score, streak, and money saved under `@rlmx_*` keys.

### demo.ts

Demo data provider. Exports:

- `demoUser` -- Cedric, Pro tier, level 14, 14-day streak, 12/50 achievements
- `demoAgents` -- 7 collected agents (Penny, Vitalis, Chronos, Sentinel, Savvy, and others) at levels 5-8

## Hooks

### useVoice

Manages voice interaction lifecycle:

- `isListening` / `isProcessing` state
- `transcript` text
- `agentProgress` array of per-agent task status
- `startListening()` / `stopListening()` controls

### useEngagement

Tracks engagement metrics:

- `lifeScore` (73 in demo)
- `moneySaved` ($847 in demo)
- `streak` (14 days in demo)
- Domain scores with trend data

### useAgents

Agent collection and marketplace state:

- `agents` -- collected agent list
- `marketplaceAgents` -- available agents to install
- Agent CRUD operations

## Context

### AppContext (`src/context/AppContext.tsx`)

Global React context using `useReducer`. Provides `AppState` containing:

- `user: UserProfile`
- `agents: Agent[]`
- `lifeScore`, `moneySaved`, `streak`
- `isConnected` (server connection status)
- Domain scores and briefing items

Wrapped around the entire app in `App.tsx` via `<AppProvider>`.

## Navigation

Bottom tab navigator with 5 tabs:

| Tab      | Icon        | Screen          |
|----------|-------------|-----------------|
| Home     | `home`      | HomeScreen      |
| Agents   | `smart-toy` | AgentsScreen    |
| Voice    | `mic`       | VoiceScreen     |
| Insights | `insights`  | InsightsScreen  |
| Profile  | `person`    | ProfileScreen   |

Stack navigator wraps the tabs and adds three push screens:

- `Marketplace` (from AgentsScreen)
- `AgentDetail` (from MarketplaceScreen)
- `AgentCollection` (from AgentsScreen)

Icons use `react-native-vector-icons/MaterialIcons`.

## Theme

Dark theme with design tokens matching the RLMX web dashboard:

```typescript
// Core palette
background: '#0a0e1a'
surface:    '#141824'
cyan:       '#00e5ff'    // primary accent
violet:     '#a855f7'    // secondary accent
green:      '#22c55e'    // success / Finance
pink:       '#ec4899'    // Health
amber:      '#f59e0b'    // warnings / Safety
red:        '#ef4444'    // errors
text:       '#e2e8f0'    // primary text
textSecondary: '#94a3b8' // secondary text
textMuted:  '#64748b'    // disabled / muted

// Domain colors
Finance:    green
Health:     pink
Time:       cyan
Safety:     amber
Career:     '#3b82f6'
Education:  violet
// ... 14 domains total
```

Spacing scale: `xs=4, sm=8, md=16, lg=24, xl=32, xxl=48`

Border radius: `sm=8, md=12, lg=16, xl=24, full=999`

Font sizes: `xs=10, sm=12, md=14, lg=16, xl=20, xxl=28, hero=40`

## Demo Mode

The app works fully offline with demo data. No server connection is required to explore all screens. Demo state includes:

- **User**: Cedric, Pro tier, Level 14, 2340/3000 XP
- **Agents**: 7 collected agents at levels 5-8 across Finance, Health, Time, and Safety domains
- **Life Score**: 73
- **Money Saved**: $847
- **Streak**: 14 days (longest: 21)
- **Achievements**: 12 of 50 unlocked
- **Voice**: Simulated intent parsing with 6 persona responses

The connection dot in the header turns green when the MCP server is reachable, gray when offline.

## Building

### Debug APK

```bash
cd mobile/android

# Ensure JDK 21
export JAVA_HOME=/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home

./gradlew assembleDebug
```

Output: `android/app/build/outputs/apk/debug/app-debug.apk`

### Release APK

```bash
cd mobile/android
export JAVA_HOME=/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home

./gradlew assembleRelease
```

Output: `android/app/build/outputs/apk/release/app-release.apk`

Note: Release builds require signing configuration in `android/app/build.gradle`.

## Connecting to RLMX Server

The mobile app connects to two endpoints on the Rust backend:

| Protocol   | URL                      | Purpose                              |
|------------|--------------------------|--------------------------------------|
| HTTP       | `http://<host>:3000/mcp` | MCP JSON-RPC 2.0 tool calls         |
| WebSocket  | `ws://<host>:3001`       | Real-time SwarmEvent stream          |

### Starting the server

```bash
# From the repo root
cargo run -p rlmx-cli -- serve --port 3000
```

### Connecting from a physical device

The default base URL is `http://localhost:3000`, which works for emulators. For a physical device on the same network, update `MCP_BASE` in `src/services/api.ts` and the WebSocket URL in `src/services/websocket.ts` to your machine's LAN IP:

```typescript
const MCP_BASE = 'http://192.168.x.x:3000';
```

Alternatively, use `adb reverse` to forward ports from the device to localhost:

```bash
adb reverse tcp:3000 tcp:3000
adb reverse tcp:3001 tcp:3001
```

## Key Dependencies

| Package                         | Version  | Purpose                                 |
|---------------------------------|----------|-----------------------------------------|
| react-native                    | 0.84.1   | Core framework                          |
| react                           | 19.2.3   | UI library                              |
| @react-navigation/bottom-tabs   | 7.15.5   | Tab navigation                          |
| @react-navigation/stack          | 7.8.5    | Stack navigation                        |
| react-native-reanimated         | 4.2.2    | Animations (pulse, counters)            |
| react-native-gesture-handler    | 2.30.0   | Touch gestures                          |
| react-native-vector-icons       | 10.3.0   | MaterialIcons icon set                  |
| react-native-linear-gradient    | 2.8.3    | Gradient backgrounds on cards           |
| @react-native-async-storage     | 1.24.0   | Local key-value persistence             |
| react-native-safe-area-context  | 5.7.0    | Safe area insets                        |
| react-native-screens            | 4.24.0   | Native navigation screens               |
| react-native-worklets           | 0.7.4    | Worklet runtime for reanimated          |

## Troubleshooting

### Java version mismatch

**Symptom**: Gradle build fails with `Unsupported class file major version` or `Could not initialize class org.codehaus.groovy.vmplugin.v8.Java8`.

**Fix**: React Native 0.84.1 requires JDK 21. Java 26 is not compatible.

```bash
# Check current version
java -version

# Switch to JDK 21
export JAVA_HOME=/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home
```

### Gradle cache corruption

**Symptom**: Build fails with cryptic Gradle errors after switching JDK versions or updating dependencies.

**Fix**:

```bash
cd mobile/android
./gradlew clean
rm -rf ~/.gradle/caches/
rm -rf .gradle/
cd .. && npm install
cd android && ./gradlew assembleDebug
```

### Metro bundler issues

**Symptom**: Red screen with "Unable to resolve module" or stale bundle.

**Fix**:

```bash
cd mobile
npx react-native start --reset-cache
```

### ADB device not found

**Symptom**: `adb install` fails with "no devices/emulators found".

**Fix**:

```bash
# Check connected devices
adb devices

# For physical device: enable USB debugging in Developer Options
# For emulator: start an AVD from Android Studio
```

### react-native-vector-icons not showing

**Symptom**: Icons render as empty boxes or question marks.

**Fix**: Ensure the fonts are linked in `android/app/build.gradle`:

```groovy
apply from: file("../../node_modules/react-native-vector-icons/fonts.gradle")
```

Then rebuild:

```bash
cd mobile/android && ./gradlew clean assembleDebug
```
