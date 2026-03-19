use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Types of lock screen / home screen widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WidgetType {
    /// Active agent count, pending tasks, last action.
    AgentStatus,
    /// Cumulative savings counter with green pulse animation.
    MoneySavedCounter,
    /// Tap-to-speak from lock screen (iOS Live Activity / Android Widget).
    QuickVoice,
    /// Composite life score card.
    LifeScoreCard,
    /// Daily briefing summary.
    DailyBriefing,
    /// Current streak tracker.
    StreakTracker,
}

/// Data payload for a widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub widget_type: WidgetType,
    pub data: serde_json::Value,
    pub last_updated: DateTime<Utc>,
}

/// Manages lock screen and home screen widget state.
///
/// Invariant 8: widgets update at most once per 60 seconds.
#[derive(Debug)]
pub struct WidgetManager {
    pub active_widgets: Vec<Widget>,
    pub last_update: HashMap<WidgetType, DateTime<Utc>>,
    pub update_interval: Duration,
}

impl WidgetManager {
    /// Create a new widget manager with the default 60s minimum update interval.
    pub fn new() -> Self {
        Self {
            active_widgets: Vec::new(),
            last_update: HashMap::new(),
            update_interval: Duration::from_secs(60),
        }
    }

    /// Set the minimum update interval.
    pub fn set_update_interval(&mut self, interval: Duration) {
        // Enforce minimum of 60 seconds (invariant 8).
        if interval < Duration::from_secs(60) {
            self.update_interval = Duration::from_secs(60);
        } else {
            self.update_interval = interval;
        }
    }

    /// Check whether a widget type is due for an update.
    pub fn can_update(&self, widget_type: WidgetType) -> bool {
        match self.last_update.get(&widget_type) {
            Some(last) => {
                let elapsed = Utc::now()
                    .signed_duration_since(*last)
                    .to_std()
                    .unwrap_or(Duration::ZERO);
                elapsed >= self.update_interval
            }
            None => true, // Never updated — allow.
        }
    }

    /// Update a widget's data.
    pub fn update_widget(
        &mut self,
        widget_type: WidgetType,
        data: serde_json::Value,
    ) -> bool {
        if !self.can_update(widget_type) {
            return false;
        }

        let now = Utc::now();
        self.last_update.insert(widget_type, now);

        if let Some(widget) = self
            .active_widgets
            .iter_mut()
            .find(|w| w.widget_type == widget_type)
        {
            widget.data = data;
            widget.last_updated = now;
        } else {
            self.active_widgets.push(Widget {
                widget_type,
                data,
                last_updated: now,
            });
        }

        tracing::debug!(widget = ?widget_type, "widget updated");
        true
    }

    /// Get widget data by type.
    pub fn get_widget(&self, widget_type: WidgetType) -> Option<&Widget> {
        self.active_widgets
            .iter()
            .find(|w| w.widget_type == widget_type)
    }

    /// Generate agent status widget data.
    pub fn agent_status_data(active_count: usize, pending_tasks: usize, last_action: &str) -> serde_json::Value {
        serde_json::json!({
            "active_agents": active_count,
            "pending_tasks": pending_tasks,
            "last_action": last_action,
        })
    }

    /// Generate money saved widget data.
    pub fn money_saved_data(total_cents: u64, today_cents: u64) -> serde_json::Value {
        serde_json::json!({
            "total_dollars": total_cents as f64 / 100.0,
            "today_dollars": today_cents as f64 / 100.0,
        })
    }

    /// Generate quick voice widget data.
    pub fn quick_voice_data(listening: bool) -> serde_json::Value {
        serde_json::json!({
            "listening": listening,
            "tap_to_speak": true,
        })
    }
}

impl Default for WidgetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_widget() {
        let mut wm = WidgetManager::new();
        // First update should always succeed.
        assert!(wm.update_widget(WidgetType::AgentStatus, serde_json::json!({"active": 3})));
        assert!(wm.get_widget(WidgetType::AgentStatus).is_some());
    }

    #[test]
    fn test_minimum_interval_enforcement() {
        let mut wm = WidgetManager::new();
        wm.set_update_interval(Duration::from_secs(10)); // Below minimum.
        assert_eq!(wm.update_interval, Duration::from_secs(60));
    }

    #[test]
    fn test_agent_status_data() {
        let data = WidgetManager::agent_status_data(5, 3, "email triage");
        assert_eq!(data["active_agents"], 5);
    }

    #[test]
    fn test_money_saved_data() {
        let data = WidgetManager::money_saved_data(1500, 200);
        assert_eq!(data["total_dollars"], 15.0);
        assert_eq!(data["today_dollars"], 2.0);
    }
}
