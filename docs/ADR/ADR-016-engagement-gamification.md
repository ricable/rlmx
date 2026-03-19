# ADR-016: Engagement and Gamification System

| Field    | Value                     |
|----------|---------------------------|
| Status   | Implemented               |
| Date     | 2026-03-19                |
| Authors  | Cedric                    |
| Replaces | —                         |

## Context

The PRD mandates an "addictive free tier" with engagement loops that drive daily usage and conversion. Current RLMX has no user-facing engagement mechanics — it's pure infrastructure. The PRD requires: Life Score (0-100 daily score across Finance/Health/Time/Safety), Money Saved counter (real-time cumulative savings), streak mechanics (daily engagement with progressive rewards), and agent gamification (collection grid, levels, achievements). These must feel genuine (rewards tied to real value), not exploitative.

## Decision

### Life Score System

A daily 0-100 composite score across 4 domains:
- **Finance** (0-100): bill optimization %, savings rate, unnecessary spend detected
- **Health** (0-100): medication adherence, appointment compliance, health trajectory trend
- **Time** (0-100): calendar efficiency, email response time, task completion rate
- **Safety** (0-100): privacy violations caught, scams blocked, TOS issues flagged

Computed by `rlmx-cognitive` SONA integration:
- Each agent reports domain-specific metrics via `StateMutate` syscall
- `LifeScoreEngine` in `rlmx-phone` aggregates per-domain scores
- Sparkline trends stored for 30/90/365 day views
- Always room for improvement (score asymptotically approaches 100)
- Never discouraging (score floor of 30 for active users)
- Each score drop includes specific actionable suggestion

### Money Saved Counter

Real-time cumulative savings tracking:
- Sources: bill negotiations, price comparisons, subscription cancellations, insurance optimizations
- Each saving event is `ProofSeal`-verified (witness chain proves the saving is real)
- Display: always visible in app header + lock screen widget
- Cumulative since join date — number only goes up
- Social context: percentile ranking among local users (anonymized)
- Breakdown view: per-agent, per-domain, per-month

### Streak Mechanics

Daily engagement streaks with progressive rewards:

| Streak | Reward | Type |
|--------|--------|------|
| 3-day | "Getting Started" badge | Cosmetic |
| 7-day | Premium agent unlocked 48h | Trial |
| 14-day | Choose 1 premium agent for 7 days | Trial |
| 30-day | Custom voice persona | Cosmetic |
| 60-day | Priority federated learning | Functional |
| 90-day | Free month of Plus | Conversion |
| 365-day | "Founding Member" + permanent 10% discount | Loyalty |

Streak counted by daily briefing interaction (voice or tap). Missed day = streak resets but cumulative total preserved. Grace period: 1 "freeze" per 30 days.

### Agent Gamification

- **Agent Levels (1-10)**: agents level up as SONA learns user preferences. Level reflects real capability improvement, not cosmetic.
- **Collection Grid**: Pokemon-style grid of all available agents (52 at launch). Installed agents show in color, uninstalled are greyed silhouettes.
- **Achievements (50+)**: "First Save", "Bill Slayer", "Health Nut", "Privacy Pro", "Tax Wizard", etc.
- **User Level (1-100)**: XP earned from agent interactions, savings found, achievements unlocked
- **Opt-in Leaderboards**: city-level, friend-group, global

### Variable Reward Schedule

Based on Nir Eyal's Hook Model, adapted for genuine value:
- Rewards are REAL (actual money saved, not points)
- Timing varies (immediate, delayed, compounding) — optimized for engagement
- Value increases over time (more data = more savings discovered)
- No dark patterns: user can disable all notifications and gamification

### Data Model

```rust
pub struct UserEngagement {
    life_score: LifeScore,
    money_saved: MoneySaved,
    streak: StreakState,
    achievements: Vec<Achievement>,
    agent_levels: HashMap<AgentId, u8>,  // 1-10
    user_level: u16,                      // 1-100
    xp: u64,
}

pub struct LifeScore {
    finance: f32,    // 0-100
    health: f32,
    time: f32,
    safety: f32,
    composite: f32,  // weighted average
    history: Vec<(DateTime, f32)>,  // daily snapshots
}

pub struct MoneySaved {
    total_cents: u64,
    today_cents: u64,
    events: Vec<SavingsEvent>,  // ProofSeal-verified
}
```

### Storage

- Engagement state stored on-device in `rlmx-phone` local storage
- Synced to home hub (Zone C) via gossip for backup
- Leaderboard data only: anonymized scores sent to cloud for ranking
- No engagement data sold or shared

## Consequences

### Positive

- Genuine value tied to engagement (money saved is real, levels reflect real learning)
- 90-day streak → free Plus month is a powerful conversion trigger
- Life Score creates daily reason to check the app
- Collection grid drives marketplace exploration and agent installation

### Negative

- Gamification can feel manipulative if not carefully tuned
- Streak mechanics may create anxiety ("I can't miss a day")
- Leaderboards can discourage users who start late

### Risks

- Over-gamification could undermine trust ("this feels like a game, not a tool")
- False savings claims would destroy credibility — ProofSeal verification is critical
- Regulatory risk: some jurisdictions may classify streak rewards as gambling mechanics

## References

- ADR-013: Phone as Command Center (widgets, notifications)
- ADR-014: Agent Marketplace (agent collection, levels)
- PRD Section 7: Addictive Free Tier

## Implementation Notes

Implemented in `crates/rlmx-phone/src/engagement.rs`. LifeScore (0-100 across Finance/Health/Time/Safety, floor at 30). MoneySavedCounter (cumulative, ProofSeal-verified). StreakTracker (3/7/14/30/60/90/365 day milestones with rewards, 1 freeze per 30 days). AgentCollection (levels 1-10, XP system). 50 achievements defined. Hook: `scripts/hooks/on-streak.sh`, `scripts/hooks/on-agent-level.sh`. CLI: `cargo run -p rlmx-cli -- engagement score|savings|streak|achievements`
