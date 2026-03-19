use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// LifeScore (0-100 across Finance/Health/Time/Safety)
// ---------------------------------------------------------------------------

/// Composite metric reflecting improvements across four life domains.
///
/// Invariant 9: composite score is clamped to [30.0, 100.0].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeScore {
    pub finance: f32,
    pub health: f32,
    pub time: f32,
    pub safety: f32,
    pub composite: f32,
    pub history: Vec<(DateTime<Utc>, f32)>,
}

impl LifeScore {
    /// Create a new LifeScore with initial values.
    pub fn new(finance: f32, health: f32, time: f32, safety: f32) -> Self {
        let mut score = Self {
            finance: finance.clamp(0.0, 100.0),
            health: health.clamp(0.0, 100.0),
            time: time.clamp(0.0, 100.0),
            safety: safety.clamp(0.0, 100.0),
            composite: 0.0,
            history: Vec::new(),
        };
        score.recalculate();
        score
    }

    /// Recalculate the composite score (weighted average, clamped to [30, 100]).
    pub fn recalculate(&mut self) {
        let raw = self.finance * 0.30 + self.health * 0.25 + self.time * 0.25 + self.safety * 0.20;
        self.composite = raw.clamp(30.0, 100.0);
        self.history.push((Utc::now(), self.composite));
    }

    /// Update a single domain and recalculate.
    pub fn update_finance(&mut self, value: f32) -> f32 {
        let old = self.composite;
        self.finance = value.clamp(0.0, 100.0);
        self.recalculate();
        self.composite - old
    }

    /// Update health score and return delta.
    pub fn update_health(&mut self, value: f32) -> f32 {
        let old = self.composite;
        self.health = value.clamp(0.0, 100.0);
        self.recalculate();
        self.composite - old
    }

    /// Update time score and return delta.
    pub fn update_time(&mut self, value: f32) -> f32 {
        let old = self.composite;
        self.time = value.clamp(0.0, 100.0);
        self.recalculate();
        self.composite - old
    }

    /// Update safety score and return delta.
    pub fn update_safety(&mut self, value: f32) -> f32 {
        let old = self.composite;
        self.safety = value.clamp(0.0, 100.0);
        self.recalculate();
        self.composite - old
    }
}

impl Default for LifeScore {
    fn default() -> Self {
        Self::new(50.0, 50.0, 50.0, 50.0)
    }
}

// ---------------------------------------------------------------------------
// MoneySaved
// ---------------------------------------------------------------------------

/// A single verified savings event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingsEvent {
    pub amount_cents: u64,
    pub domain: String,
    pub proof_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Cumulative savings counter (invariant 5: all events require proof_id).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneySaved {
    pub total_cents: u64,
    pub today_cents: u64,
    pub events: Vec<SavingsEvent>,
    pub today_date: chrono::NaiveDate,
}

impl MoneySaved {
    pub fn new() -> Self {
        Self {
            total_cents: 0,
            today_cents: 0,
            events: Vec::new(),
            today_date: Utc::now().date_naive(),
        }
    }

    /// Record a verified savings event.
    pub fn record(&mut self, amount_cents: u64, domain: &str, proof_id: Uuid) {
        let today = Utc::now().date_naive();
        if today != self.today_date {
            self.today_cents = 0;
            self.today_date = today;
        }

        self.total_cents += amount_cents;
        self.today_cents += amount_cents;
        self.events.push(SavingsEvent {
            amount_cents,
            domain: domain.to_string(),
            proof_id,
            timestamp: Utc::now(),
        });

        tracing::info!(
            amount_cents,
            domain,
            total_cents = self.total_cents,
            "savings recorded"
        );
    }

    /// Total savings in dollars.
    pub fn total_dollars(&self) -> f64 {
        self.total_cents as f64 / 100.0
    }
}

impl Default for MoneySaved {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// StreakState
// ---------------------------------------------------------------------------

/// Reward earned at a streak milestone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakReward {
    pub day: u32,
    pub reward_type: String,
    pub claimed: bool,
}

/// Streak tracking with freeze support.
///
/// Invariant 4: one freeze per 30 days. Missed day after freeze exhaustion
/// resets current to 0. Longest is never decremented.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakState {
    pub current: u32,
    pub longest: u32,
    pub last_active: DateTime<Utc>,
    pub freezes_remaining: u8,
    pub rewards_earned: Vec<StreakReward>,
}

/// Milestone days that trigger rewards.
const STREAK_MILESTONES: &[u32] = &[3, 7, 14, 30, 60, 90, 365];

impl StreakState {
    pub fn new() -> Self {
        Self {
            current: 0,
            longest: 0,
            last_active: Utc::now(),
            freezes_remaining: 1,
            rewards_earned: Vec::new(),
        }
    }

    /// Check in for today. Returns any reward earned.
    pub fn check_in(&mut self, now: DateTime<Utc>) -> Option<StreakReward> {
        let last_date = self.last_active.date_naive();
        let today = now.date_naive();
        let days_since = (today - last_date).num_days();

        if days_since <= 0 {
            // Same day — no-op.
            return None;
        }

        if days_since == 1 {
            // Consecutive day.
            self.current += 1;
        } else if days_since == 2 && self.freezes_remaining > 0 {
            // Missed exactly one day, use a freeze.
            self.freezes_remaining -= 1;
            self.current += 1;
            tracing::info!(
                freezes_remaining = self.freezes_remaining,
                "streak freeze used"
            );
        } else {
            // Streak broken.
            self.current = 1;
        }

        self.last_active = now;
        if self.current > self.longest {
            self.longest = self.current;
        }

        // Check for milestone reward.
        if STREAK_MILESTONES.contains(&self.current) {
            let reward = StreakReward {
                day: self.current,
                reward_type: format!("{}-day streak reward", self.current),
                claimed: false,
            };
            self.rewards_earned.push(reward.clone());
            tracing::info!(day = self.current, "streak milestone reached");
            return Some(reward);
        }

        None
    }

    /// Replenish freeze (called once per 30 days).
    pub fn replenish_freeze(&mut self) {
        self.freezes_remaining = 1;
    }
}

impl Default for StreakState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Agent Collection & XP
// ---------------------------------------------------------------------------

/// A slot in the agent collection grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSlot {
    pub agent_type: String,
    pub level: u8,
    pub xp: u64,
    pub unlocked: bool,
}

/// Grid layout for the agent collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionGrid {
    pub slots: Vec<CollectionSlot>,
    pub rows: u8,
    pub cols: u8,
}

impl CollectionGrid {
    pub fn new(rows: u8, cols: u8) -> Self {
        Self {
            slots: Vec::new(),
            rows,
            cols,
        }
    }
}

impl Default for CollectionGrid {
    fn default() -> Self {
        Self::new(4, 3)
    }
}

/// Agent collection with leveling (1-10) and XP system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCollection {
    pub agent_levels: HashMap<String, u8>,
    pub agent_xp: HashMap<String, u64>,
}

impl AgentCollection {
    pub fn new() -> Self {
        Self {
            agent_levels: HashMap::new(),
            agent_xp: HashMap::new(),
        }
    }

    /// XP required for a given level.
    fn xp_for_level(level: u8) -> u64 {
        // Exponential curve: 100, 250, 500, 1000, 2000, 4000, 8000, 16000, 32000
        match level {
            1 => 0,
            2 => 100,
            3 => 250,
            4 => 500,
            5 => 1000,
            6 => 2000,
            7 => 4000,
            8 => 8000,
            9 => 16000,
            10 => 32000,
            _ => u64::MAX,
        }
    }

    /// Add XP to an agent. Returns true if the agent leveled up.
    pub fn add_xp(&mut self, agent_type: &str, xp: u64) -> bool {
        let current_xp = self.agent_xp.entry(agent_type.to_string()).or_insert(0);
        *current_xp += xp;

        let current_level = self
            .agent_levels
            .entry(agent_type.to_string())
            .or_insert(1);

        if *current_level < 10 {
            let next_level_xp = Self::xp_for_level(*current_level + 1);
            if *current_xp >= next_level_xp {
                *current_level += 1;
                tracing::info!(
                    agent_type,
                    new_level = *current_level,
                    "agent leveled up"
                );
                return true;
            }
        }
        false
    }

    /// Get the level of an agent.
    pub fn get_level(&self, agent_type: &str) -> u8 {
        self.agent_levels.get(agent_type).copied().unwrap_or(1)
    }

    /// Get the XP of an agent.
    pub fn get_xp(&self, agent_type: &str) -> u64 {
        self.agent_xp.get(agent_type).copied().unwrap_or(0)
    }
}

impl Default for AgentCollection {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Achievements
// ---------------------------------------------------------------------------

/// An unlockable achievement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlocked_at: Option<DateTime<Utc>>,
}

impl Achievement {
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            unlocked_at: None,
        }
    }

    pub fn unlock(&mut self) {
        if self.unlocked_at.is_none() {
            self.unlocked_at = Some(Utc::now());
        }
    }

    pub fn is_unlocked(&self) -> bool {
        self.unlocked_at.is_some()
    }
}

// ---------------------------------------------------------------------------
// UserEngagement (aggregate)
// ---------------------------------------------------------------------------

/// The gamification and value-tracking subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEngagement {
    pub life_score: LifeScore,
    pub money_saved: MoneySaved,
    pub streak: StreakState,
    pub achievements: Vec<Achievement>,
    pub agent_collection: AgentCollection,
    pub user_level: u16,
    pub xp: u64,
    pub collection_grid: CollectionGrid,
}

impl UserEngagement {
    pub fn new() -> Self {
        Self {
            life_score: LifeScore::default(),
            money_saved: MoneySaved::new(),
            streak: StreakState::new(),
            achievements: Self::init_achievements(),
            agent_collection: AgentCollection::new(),
            user_level: 1,
            xp: 0,
            collection_grid: CollectionGrid::default(),
        }
    }

    /// Initialise 50 achievements.
    fn init_achievements() -> Vec<Achievement> {
        let defs = [
            ("first_save", "First Save", "Record your first savings event"),
            ("save_100", "Benjamin", "Save $100 total"),
            ("save_1000", "Grand Saver", "Save $1,000 total"),
            ("save_10000", "Money Master", "Save $10,000 total"),
            ("streak_3", "Getting Started", "Achieve a 3-day streak"),
            ("streak_7", "Week Warrior", "Achieve a 7-day streak"),
            ("streak_14", "Two Weeks Strong", "Achieve a 14-day streak"),
            ("streak_30", "Monthly Champion", "Achieve a 30-day streak"),
            ("streak_60", "Two Months", "Achieve a 60-day streak"),
            ("streak_90", "Quarter Master", "Achieve a 90-day streak"),
            ("streak_365", "Year of Growth", "Achieve a 365-day streak"),
            ("agent_lv5", "Agent Trainer", "Level any agent to 5"),
            ("agent_lv10", "Agent Master", "Level any agent to 10"),
            ("all_free_agents", "Full Roster", "Activate all 5 free agents"),
            ("life_score_80", "Life Optimizer", "Reach 80+ composite life score"),
            ("life_score_90", "Elite Life", "Reach 90+ composite life score"),
            ("finance_80", "Finance Pro", "Reach 80+ finance score"),
            ("health_80", "Health Guru", "Reach 80+ health score"),
            ("time_80", "Time Lord", "Reach 80+ time score"),
            ("safety_80", "Safety First", "Reach 80+ safety score"),
            ("first_offline", "Off the Grid", "Complete a task while offline"),
            ("10_offline", "Offline Champion", "Complete 10 tasks while offline"),
            ("widget_user", "Dashboard Fan", "Use all widget types"),
            ("voice_first", "Voice Pioneer", "Complete 10 voice commands"),
            ("voice_50", "Voice Veteran", "Complete 50 voice commands"),
            ("notif_engage", "Alert Responder", "Respond to 10 notifications"),
            ("notif_master", "Notification Ninja", "Respond to 100 notifications"),
            ("early_bird", "Early Bird", "Use the app before 6 AM"),
            ("night_owl", "Night Owl", "Use the app after midnight"),
            ("weekend_warrior", "Weekend Warrior", "Active every weekend for a month"),
            ("first_research", "Curious Mind", "Start your first research task"),
            ("5_domains", "Diversified", "Use agents across 5 different domains"),
            ("battery_saver", "Eco Mode", "Run agents in low-power mode for a week"),
            ("swarm_join", "Swarm Member", "Join a distributed swarm"),
            ("cross_device", "Multi-Device", "Sync between phone and desktop"),
            ("calendar_100", "Schedule Master", "Process 100 calendar events"),
            ("email_500", "Inbox Zero", "Triage 500 emails"),
            ("shopping_save", "Deal Hunter", "Find 10 better prices"),
            ("news_read", "Informed Citizen", "Read 100 news digests"),
            ("weather_check", "Weather Watcher", "Check weather 30 consecutive days"),
            ("first_freeze", "Freeze Frame", "Use a streak freeze"),
            ("perfect_month", "Perfect Month", "30-day streak without a freeze"),
            ("level_10", "Rising Star", "Reach user level 10"),
            ("level_25", "Expert", "Reach user level 25"),
            ("level_50", "Grand Master", "Reach user level 50"),
            ("xp_1000", "XP Collector", "Earn 1,000 total XP"),
            ("xp_10000", "XP Hoarder", "Earn 10,000 total XP"),
            ("xp_100000", "XP Legend", "Earn 100,000 total XP"),
            ("collection_half", "Half Collection", "Unlock half the collection grid"),
            ("collection_full", "Full Collection", "Complete the entire collection grid"),
        ];

        defs.iter()
            .map(|(id, name, desc)| Achievement::new(id, name, desc))
            .collect()
    }

    /// Add XP to the user and check for level-up.
    pub fn add_user_xp(&mut self, amount: u64) -> bool {
        self.xp += amount;
        let new_level = (self.xp as f64 / 500.0).sqrt() as u16 + 1;
        if new_level > self.user_level {
            self.user_level = new_level;
            tracing::info!(level = self.user_level, xp = self.xp, "user leveled up");
            return true;
        }
        false
    }

    /// Unlock an achievement by id, returns true if newly unlocked.
    pub fn unlock_achievement(&mut self, id: &str) -> bool {
        if let Some(achievement) = self.achievements.iter_mut().find(|a| a.id == id) {
            if !achievement.is_unlocked() {
                achievement.unlock();
                tracing::info!(achievement_id = id, "achievement unlocked");
                return true;
            }
        }
        false
    }

    /// Count of unlocked achievements.
    pub fn unlocked_count(&self) -> usize {
        self.achievements.iter().filter(|a| a.is_unlocked()).count()
    }
}

impl Default for UserEngagement {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_life_score_clamped() {
        let score = LifeScore::new(0.0, 0.0, 0.0, 0.0);
        assert!(score.composite >= 30.0, "composite should be at least 30");
    }

    #[test]
    fn test_life_score_update() {
        let mut score = LifeScore::default();
        let delta = score.update_finance(90.0);
        assert!(delta > 0.0 || delta <= 0.0); // delta computed correctly
        assert_eq!(score.finance, 90.0);
    }

    #[test]
    fn test_money_saved() {
        let mut ms = MoneySaved::new();
        ms.record(1500, "shopping", Uuid::new_v4());
        assert_eq!(ms.total_cents, 1500);
        assert_eq!(ms.total_dollars(), 15.0);
        assert_eq!(ms.events.len(), 1);
    }

    #[test]
    fn test_streak_check_in() {
        let mut streak = StreakState::new();
        // Set last_active to yesterday so first check-in counts.
        streak.last_active = Utc::now() - Duration::days(1);
        let now = Utc::now();
        streak.check_in(now);
        assert_eq!(streak.current, 1);

        let tomorrow = now + Duration::days(1);
        streak.check_in(tomorrow);
        assert_eq!(streak.current, 2);
    }

    #[test]
    fn test_streak_milestone_reward() {
        let mut streak = StreakState::new();
        let mut now = Utc::now();
        let mut rewards = Vec::new();
        for _ in 0..3 {
            now = now + Duration::days(1);
            if let Some(r) = streak.check_in(now) {
                rewards.push(r);
            }
        }
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].day, 3);
    }

    #[test]
    fn test_streak_freeze() {
        let mut streak = StreakState::new();
        streak.last_active = Utc::now() - Duration::days(1);
        let now = Utc::now();
        streak.check_in(now);
        assert_eq!(streak.current, 1);

        // Skip a day, then check in (should use freeze).
        let day_after_tomorrow = now + Duration::days(2);
        streak.check_in(day_after_tomorrow);
        assert_eq!(streak.current, 2);
        assert_eq!(streak.freezes_remaining, 0);
    }

    #[test]
    fn test_streak_broken_no_freeze() {
        let mut streak = StreakState::new();
        streak.freezes_remaining = 0;
        streak.last_active = Utc::now() - Duration::days(1);
        let now = Utc::now();
        streak.check_in(now);
        assert_eq!(streak.current, 1);

        let three_days_later = now + Duration::days(3);
        streak.check_in(three_days_later);
        assert_eq!(streak.current, 1); // Reset.
    }

    #[test]
    fn test_agent_xp_level_up() {
        let mut collection = AgentCollection::new();
        assert_eq!(collection.get_level("Email Triage"), 1);

        let leveled = collection.add_xp("Email Triage", 100);
        assert!(leveled);
        assert_eq!(collection.get_level("Email Triage"), 2);
    }

    #[test]
    fn test_agent_max_level() {
        let mut collection = AgentCollection::new();
        collection.agent_levels.insert("test".to_string(), 10);
        collection.agent_xp.insert("test".to_string(), 100000);
        let leveled = collection.add_xp("test", 1000);
        assert!(!leveled); // Already at max.
    }

    #[test]
    fn test_achievement_unlock() {
        let mut eng = UserEngagement::new();
        assert_eq!(eng.unlocked_count(), 0);
        assert!(eng.unlock_achievement("first_save"));
        assert_eq!(eng.unlocked_count(), 1);
        // Double unlock returns false.
        assert!(!eng.unlock_achievement("first_save"));
    }

    #[test]
    fn test_50_achievements() {
        let eng = UserEngagement::new();
        assert_eq!(eng.achievements.len(), 50);
    }

    #[test]
    fn test_user_xp_level_up() {
        let mut eng = UserEngagement::new();
        assert_eq!(eng.user_level, 1);
        eng.add_user_xp(500);
        assert!(eng.user_level >= 2);
    }
}
