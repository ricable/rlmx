//! Voice session management (DDD-008 aggregate root).
//!
//! `VoiceSession` is the aggregate root: it owns the consistency boundary
//! for a single voice interaction lifecycle from wake word through response
//! delivery. All mutations to turns, intent state, and emotion tracking
//! flow through the session.

use chrono::{DateTime, Utc};
pub use rlmx_kernel::{Intent, LifeDomain, ResponseMode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Direction of a conversation turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnDirection {
    /// User speaking to the system.
    UserToSystem,
    /// System responding to the user.
    SystemToUser,
}

/// A single user-system exchange within a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    /// Unique identifier for this turn.
    pub id: Uuid,
    /// Direction of the exchange.
    pub direction: TurnDirection,
    /// Transcribed text (user) or generated response (system).
    pub transcript: String,
    /// Duration of the audio in milliseconds.
    pub audio_duration_ms: u64,
    /// Intents extracted from this turn (user turns only).
    pub intents: Vec<Intent>,
    /// The response text (system turns only).
    pub response: Option<String>,
    /// When this turn occurred.
    pub timestamp: DateTime<Utc>,
}

/// The aggregate root for a voice interaction (DDD-008).
///
/// A session is the unit of resource allocation (audio buffer, STT context,
/// TTS stream) and the unit of cleanup on timeout (5 minutes of inactivity).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSession {
    /// Unique session identifier.
    pub id: Uuid,
    /// On-device speaker identification (if recognized).
    pub speaker_id: Option<Uuid>,
    /// Current response delivery mode.
    pub mode: ResponseMode,
    /// Ordered conversation turns.
    pub turns: Vec<ConversationTurn>,
    /// Currently active (unresolved) intents.
    pub active_intents: Vec<Intent>,
    /// When the session started.
    pub started_at: DateTime<Utc>,
    /// When the last activity occurred.
    pub last_activity: DateTime<Utc>,
    /// Emotion valence trajectory over the session [-1, 1].
    pub emotion_trajectory: Vec<f32>,
}

/// Session timeout duration in milliseconds (5 minutes per DDD-008).
const SESSION_TIMEOUT_MS: u64 = 5 * 60 * 1000;

impl VoiceSession {
    /// Create a new voice session.
    pub fn new(mode: ResponseMode) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            speaker_id: None,
            mode,
            turns: Vec::new(),
            active_intents: Vec::new(),
            started_at: now,
            last_activity: now,
            emotion_trajectory: Vec::new(),
        }
    }

    /// Create a session with a known speaker.
    pub fn with_speaker(mut self, speaker_id: Uuid) -> Self {
        self.speaker_id = Some(speaker_id);
        self
    }

    /// Add a user turn with transcript and extracted intents.
    pub fn add_user_turn(&mut self, transcript: String, intents: Vec<Intent>) {
        let turn = ConversationTurn {
            id: Uuid::new_v4(),
            direction: TurnDirection::UserToSystem,
            transcript,
            audio_duration_ms: 0,
            intents: intents.clone(),
            response: None,
            timestamp: Utc::now(),
        };
        self.turns.push(turn);
        self.active_intents.extend(intents);
        self.last_activity = Utc::now();
    }

    /// Add a system response turn.
    pub fn add_system_turn(&mut self, response: String) {
        let turn = ConversationTurn {
            id: Uuid::new_v4(),
            direction: TurnDirection::SystemToUser,
            transcript: response.clone(),
            audio_duration_ms: 0,
            intents: Vec::new(),
            response: Some(response),
            timestamp: Utc::now(),
        };
        self.turns.push(turn);
        self.last_activity = Utc::now();
    }

    /// Record an emotion valence reading.
    pub fn record_emotion(&mut self, valence: f32) {
        self.emotion_trajectory.push(valence.clamp(-1.0, 1.0));
    }

    /// Resolve (remove) active intents for a given domain.
    pub fn resolve_intents(&mut self, domain: LifeDomain) {
        self.active_intents.retain(|i| i.domain != domain);
        self.last_activity = Utc::now();
    }

    /// Change the response mode (takes effect on next turn per DDD-008).
    pub fn set_mode(&mut self, mode: ResponseMode) {
        self.mode = mode;
    }

    /// Check if the session has timed out (5 minutes of inactivity).
    pub fn is_timed_out(&self) -> bool {
        let elapsed = Utc::now()
            .signed_duration_since(self.last_activity)
            .num_milliseconds();
        elapsed > SESSION_TIMEOUT_MS as i64
    }

    /// Get the number of turns in this session.
    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }

    /// Get the session duration in milliseconds.
    pub fn duration_ms(&self) -> u64 {
        Utc::now()
            .signed_duration_since(self.started_at)
            .num_milliseconds()
            .max(0) as u64
    }

    /// Get the average emotion valence for the session.
    pub fn average_emotion(&self) -> f32 {
        if self.emotion_trajectory.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.emotion_trajectory.iter().sum();
        sum / self.emotion_trajectory.len() as f32
    }
}

/// Session memory with temporal-weighted search.
///
/// Stores session summaries for long-term retrieval. In production this
/// backs into SONA PatternBank (rlmx-cognitive). The stub uses in-memory
/// storage.
#[derive(Debug, Default)]
pub struct SessionMemory {
    sessions: Vec<SessionSummary>,
}

/// Summary of a completed voice session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: Uuid,
    pub speaker_id: Option<Uuid>,
    pub turn_count: usize,
    pub duration_ms: u64,
    pub domains_touched: Vec<LifeDomain>,
    pub average_emotion: f32,
    pub ended_at: DateTime<Utc>,
}

impl SessionMemory {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }

    /// Store a completed session summary.
    pub fn store(&mut self, session: &VoiceSession) {
        let domains: Vec<LifeDomain> = session
            .turns
            .iter()
            .flat_map(|t| t.intents.iter().map(|i| i.domain))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        self.sessions.push(SessionSummary {
            session_id: session.id,
            speaker_id: session.speaker_id,
            turn_count: session.turn_count(),
            duration_ms: session.duration_ms(),
            domains_touched: domains,
            average_emotion: session.average_emotion(),
            ended_at: Utc::now(),
        });

        tracing::debug!(
            session_id = %session.id,
            turn_count = session.turn_count(),
            "session summary stored"
        );
    }

    /// Search for past sessions, weighted by recency.
    ///
    /// Returns up to `limit` sessions, most recent first. In production
    /// this uses temporal-weighted VecSearch through the kernel.
    pub fn search_recent(&self, limit: usize) -> Vec<&SessionSummary> {
        let mut sorted: Vec<&SessionSummary> = self.sessions.iter().collect();
        sorted.sort_by(|a, b| b.ended_at.cmp(&a.ended_at));
        sorted.truncate(limit);
        sorted
    }

    /// Search for sessions involving a specific speaker.
    pub fn search_by_speaker(&self, speaker_id: Uuid) -> Vec<&SessionSummary> {
        self.sessions
            .iter()
            .filter(|s| s.speaker_id == Some(speaker_id))
            .collect()
    }

    /// Get the total number of stored sessions.
    pub fn count(&self) -> usize {
        self.sessions.len()
    }
}

/// Domain events emitted by the voice pipeline (DDD-008).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoiceDomainEvent {
    SessionStarted {
        session_id: Uuid,
        speaker_id: Option<Uuid>,
        mode: ResponseMode,
    },
    SpeechDetected {
        session_id: Uuid,
        vad_confidence: f32,
    },
    TranscriptReady {
        session_id: Uuid,
        transcript: String,
        stt_confidence: f32,
        is_final: bool,
    },
    IntentsDecomposed {
        session_id: Uuid,
        intents: Vec<Intent>,
    },
    ResponseStreaming {
        session_id: Uuid,
        domain: String,
        chunk_index: u32,
    },
    SessionEnded {
        session_id: Uuid,
        turn_count: u32,
        duration_ms: u64,
    },
    ResponseModeChanged {
        session_id: Uuid,
        old_mode: ResponseMode,
        new_mode: ResponseMode,
    },
    SpeakerIdentified {
        session_id: Uuid,
        speaker_id: Uuid,
        confidence: f32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_intent(domain: LifeDomain) -> Intent {
        Intent {
            domain,
            action: "test".to_string(),
            entities: Vec::new(),
            urgency: 0.5,
            confidence: 0.8,
        }
    }

    #[test]
    fn test_session_creation() {
        let session = VoiceSession::new(ResponseMode::Multimodal);
        assert_eq!(session.mode, ResponseMode::Multimodal);
        assert!(session.turns.is_empty());
        assert!(session.active_intents.is_empty());
        assert!(session.speaker_id.is_none());
    }

    #[test]
    fn test_session_with_speaker() {
        let speaker = Uuid::new_v4();
        let session = VoiceSession::new(ResponseMode::VoiceOnly).with_speaker(speaker);
        assert_eq!(session.speaker_id, Some(speaker));
    }

    #[test]
    fn test_add_user_turn() {
        let mut session = VoiceSession::new(ResponseMode::Multimodal);
        let intents = vec![sample_intent(LifeDomain::Finance)];
        session.add_user_turn("pay my bill".to_string(), intents);
        assert_eq!(session.turn_count(), 1);
        assert_eq!(session.active_intents.len(), 1);
        assert_eq!(session.turns[0].direction, TurnDirection::UserToSystem);
    }

    #[test]
    fn test_add_system_turn() {
        let mut session = VoiceSession::new(ResponseMode::Multimodal);
        session.add_system_turn("Your bill has been paid.".to_string());
        assert_eq!(session.turn_count(), 1);
        assert_eq!(session.turns[0].direction, TurnDirection::SystemToUser);
        assert!(session.turns[0].response.is_some());
    }

    #[test]
    fn test_resolve_intents() {
        let mut session = VoiceSession::new(ResponseMode::Multimodal);
        session.add_user_turn(
            "pay bill and order food".to_string(),
            vec![
                sample_intent(LifeDomain::Finance),
                sample_intent(LifeDomain::Shopping),
            ],
        );
        assert_eq!(session.active_intents.len(), 2);

        session.resolve_intents(LifeDomain::Finance);
        assert_eq!(session.active_intents.len(), 1);
        assert_eq!(session.active_intents[0].domain, LifeDomain::Shopping);
    }

    #[test]
    fn test_emotion_tracking() {
        let mut session = VoiceSession::new(ResponseMode::VoiceOnly);
        session.record_emotion(0.5);
        session.record_emotion(-0.3);
        session.record_emotion(0.8);
        assert_eq!(session.emotion_trajectory.len(), 3);
        let avg = session.average_emotion();
        let expected = (0.5 - 0.3 + 0.8) / 3.0;
        assert!((avg - expected).abs() < 0.001);
    }

    #[test]
    fn test_emotion_clamping() {
        let mut session = VoiceSession::new(ResponseMode::VoiceOnly);
        session.record_emotion(5.0);
        session.record_emotion(-5.0);
        assert!((session.emotion_trajectory[0] - 1.0).abs() < f32::EPSILON);
        assert!((session.emotion_trajectory[1] - (-1.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_average_emotion_empty() {
        let session = VoiceSession::new(ResponseMode::Ambient);
        assert!((session.average_emotion() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_session_memory_store_and_search() {
        let mut memory = SessionMemory::new();
        let mut session = VoiceSession::new(ResponseMode::Multimodal);
        session.add_user_turn(
            "buy groceries".to_string(),
            vec![sample_intent(LifeDomain::Shopping)],
        );
        memory.store(&session);

        assert_eq!(memory.count(), 1);
        let recent = memory.search_recent(10);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].session_id, session.id);
    }

    #[test]
    fn test_session_memory_search_by_speaker() {
        let mut memory = SessionMemory::new();
        let speaker = Uuid::new_v4();

        let s1 = VoiceSession::new(ResponseMode::VoiceOnly).with_speaker(speaker);
        let s2 = VoiceSession::new(ResponseMode::VoiceOnly);
        memory.store(&s1);
        memory.store(&s2);

        let results = memory.search_by_speaker(speaker);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_session_not_timed_out_initially() {
        let session = VoiceSession::new(ResponseMode::Multimodal);
        assert!(!session.is_timed_out());
    }

    #[test]
    fn test_set_mode() {
        let mut session = VoiceSession::new(ResponseMode::VoiceOnly);
        session.set_mode(ResponseMode::Multimodal);
        assert_eq!(session.mode, ResponseMode::Multimodal);
    }

    #[test]
    fn test_voice_domain_event_serialization() {
        let event = VoiceDomainEvent::SessionStarted {
            session_id: Uuid::new_v4(),
            speaker_id: None,
            mode: ResponseMode::VoiceOnly,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("SessionStarted"));
    }

    #[test]
    fn test_session_domains_in_summary() {
        let mut memory = SessionMemory::new();
        let mut session = VoiceSession::new(ResponseMode::Multimodal);
        session.add_user_turn(
            "multi domain".to_string(),
            vec![
                sample_intent(LifeDomain::Finance),
                sample_intent(LifeDomain::Health),
            ],
        );
        memory.store(&session);
        let summaries = memory.search_recent(1);
        assert!(summaries[0].domains_touched.len() >= 2);
    }
}
