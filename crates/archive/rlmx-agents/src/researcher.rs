use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::mutation::MutationStrategy;
use crate::types::AgentId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HypothesisStatus {
    Proposed,
    Testing,
    Confirmed,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: Uuid,
    pub description: String,
    pub confidence: f64,
    pub status: HypothesisStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub hypothesis_id: Uuid,
    pub evidence: String,
    pub score: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSummary {
    pub topic: String,
    pub hypotheses_tested: usize,
    pub findings: Vec<Finding>,
    pub best_finding: Option<Finding>,
    pub duration_secs: f64,
}

/// Lifecycle status for a research objective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResearchStatus {
    Active,
    Stalled,
    Escalated,
    Completed,
}

/// Aggregate root for an evolutionary auto-research run (ADR-006).
///
/// Tracks the overall goal, generated hypotheses, experiment references,
/// the best genome found so far, and the current generation count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchObjective {
    pub id: Uuid,
    pub goal: String,
    pub hypotheses: Vec<Hypothesis>,
    pub experiments: Vec<Uuid>,
    pub best_genome: Option<MutationStrategy>,
    pub generation: u32,
    pub status: ResearchStatus,
}

impl ResearchObjective {
    pub fn new(goal: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            goal: goal.into(),
            hypotheses: Vec::new(),
            experiments: Vec::new(),
            best_genome: None,
            generation: 0,
            status: ResearchStatus::Active,
        }
    }

    /// Record an experiment id for this objective.
    pub fn add_experiment(&mut self, experiment_id: Uuid) {
        self.experiments.push(experiment_id);
    }

    /// Update the best genome if the candidate has better (lower) fitness.
    pub fn update_best_genome(&mut self, candidate: MutationStrategy) {
        let dominated = match &self.best_genome {
            Some(current) => match (candidate.fitness, current.fitness) {
                (Some(c), Some(b)) => c < b,
                (Some(_), None) => true,
                _ => false,
            },
            None => candidate.fitness.is_some(),
        };
        if dominated {
            self.best_genome = Some(candidate);
        }
    }

    /// Advance to the next generation.
    pub fn advance_generation(&mut self) {
        self.generation += 1;
    }

    /// Mark the objective as stalled.
    pub fn mark_stalled(&mut self) {
        self.status = ResearchStatus::Stalled;
    }

    /// Mark the objective as escalated (cloud Tier-3).
    pub fn mark_escalated(&mut self) {
        self.status = ResearchStatus::Escalated;
    }

    /// Mark the objective as completed.
    pub fn mark_completed(&mut self) {
        self.status = ResearchStatus::Completed;
    }
}

/// Research agent — generates hypotheses, gathers evidence, synthesizes findings.
pub struct ResearcherAgent {
    pub id: AgentId,
    pub parent: AgentId,
    pub topic: String,
    pub hypotheses: Vec<Hypothesis>,
    pub findings: Vec<Finding>,
    started_at: DateTime<Utc>,
}

impl ResearcherAgent {
    pub fn new(parent: AgentId, topic: impl Into<String>) -> Self {
        Self {
            id: AgentId::new(),
            parent,
            topic: topic.into(),
            hypotheses: Vec::new(),
            findings: Vec::new(),
            started_at: Utc::now(),
        }
    }

    /// Generate hypotheses based on the research topic.
    pub fn generate_hypotheses(&mut self) -> Vec<Hypothesis> {
        // Deterministic hypothesis generation based on topic characteristics
        let h1 = Hypothesis {
            id: Uuid::new_v4(),
            description: format!("Primary approach to {}", self.topic),
            confidence: 0.7,
            status: HypothesisStatus::Proposed,
        };
        let h2 = Hypothesis {
            id: Uuid::new_v4(),
            description: format!("Alternative approach to {}", self.topic),
            confidence: 0.5,
            status: HypothesisStatus::Proposed,
        };
        let h3 = Hypothesis {
            id: Uuid::new_v4(),
            description: format!("Null hypothesis for {}", self.topic),
            confidence: 0.3,
            status: HypothesisStatus::Proposed,
        };

        self.hypotheses = vec![h1, h2, h3];
        self.hypotheses.clone()
    }

    /// Run the research process: test each hypothesis and gather findings.
    pub async fn research(&mut self) -> Result<Vec<Finding>, String> {
        if self.hypotheses.is_empty() {
            self.generate_hypotheses();
        }

        let mut findings = Vec::new();

        for hypothesis in &mut self.hypotheses {
            hypothesis.status = HypothesisStatus::Testing;

            // Simulate evidence gathering based on hypothesis confidence
            let score = hypothesis.confidence * 0.9;
            let finding = Finding {
                hypothesis_id: hypothesis.id,
                evidence: format!("Evidence for: {}", hypothesis.description),
                score,
                timestamp: Utc::now(),
            };

            if score > 0.5 {
                hypothesis.status = HypothesisStatus::Confirmed;
            } else {
                hypothesis.status = HypothesisStatus::Rejected;
            }

            findings.push(finding);
        }

        self.findings = findings.clone();
        Ok(findings)
    }

    /// Synthesize all findings into a research summary.
    pub fn synthesize(&self) -> ResearchSummary {
        let best_finding = self
            .findings
            .iter()
            .max_by(|a, b| {
                a.score
                    .partial_cmp(&b.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned();

        let duration = Utc::now()
            .signed_duration_since(self.started_at)
            .num_milliseconds() as f64
            / 1000.0;

        ResearchSummary {
            topic: self.topic.clone(),
            hypotheses_tested: self.hypotheses.len(),
            findings: self.findings.clone(),
            best_finding,
            duration_secs: duration,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hypotheses() {
        let parent = AgentId::new();
        let mut researcher = ResearcherAgent::new(parent, "test topic");
        let hypotheses = researcher.generate_hypotheses();
        assert_eq!(hypotheses.len(), 3);
        assert!(hypotheses
            .iter()
            .all(|h| h.status == HypothesisStatus::Proposed));
    }

    #[tokio::test]
    async fn test_research() {
        let parent = AgentId::new();
        let mut researcher = ResearcherAgent::new(parent, "neural networks");
        let findings = researcher.research().await.unwrap();
        assert_eq!(findings.len(), 3);
        assert!(findings.iter().all(|f| f.score > 0.0));
    }

    #[tokio::test]
    async fn test_synthesize() {
        let parent = AgentId::new();
        let mut researcher = ResearcherAgent::new(parent, "graph algorithms");
        researcher.research().await.unwrap();
        let summary = researcher.synthesize();
        assert_eq!(summary.topic, "graph algorithms");
        assert_eq!(summary.hypotheses_tested, 3);
        assert!(summary.best_finding.is_some());
    }

    #[test]
    fn test_hypothesis_confirmation() {
        // Hypotheses with confidence > ~0.56 should be confirmed (score = conf * 0.9 > 0.5)
        let parent = AgentId::new();
        let mut researcher = ResearcherAgent::new(parent, "test");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(researcher.research()).unwrap();

        let confirmed: Vec<_> = researcher
            .hypotheses
            .iter()
            .filter(|h| h.status == HypothesisStatus::Confirmed)
            .collect();
        let rejected: Vec<_> = researcher
            .hypotheses
            .iter()
            .filter(|h| h.status == HypothesisStatus::Rejected)
            .collect();

        // conf 0.7 * 0.9 = 0.63 > 0.5 => confirmed
        // conf 0.5 * 0.9 = 0.45 <= 0.5 => rejected
        // conf 0.3 * 0.9 = 0.27 <= 0.5 => rejected
        assert_eq!(confirmed.len(), 1);
        assert_eq!(rejected.len(), 2);
    }

    // --- ResearchObjective tests ---

    #[test]
    fn test_research_objective_new() {
        let obj = ResearchObjective::new("reduce val_bpb by 5%");
        assert_eq!(obj.goal, "reduce val_bpb by 5%");
        assert_eq!(obj.status, ResearchStatus::Active);
        assert_eq!(obj.generation, 0);
        assert!(obj.hypotheses.is_empty());
        assert!(obj.experiments.is_empty());
        assert!(obj.best_genome.is_none());
    }

    #[test]
    fn test_research_objective_add_experiment() {
        let mut obj = ResearchObjective::new("test");
        let eid = Uuid::new_v4();
        obj.add_experiment(eid);
        assert_eq!(obj.experiments.len(), 1);
        assert_eq!(obj.experiments[0], eid);
    }

    #[test]
    fn test_research_objective_advance_generation() {
        let mut obj = ResearchObjective::new("test");
        obj.advance_generation();
        assert_eq!(obj.generation, 1);
        obj.advance_generation();
        assert_eq!(obj.generation, 2);
    }

    #[test]
    fn test_research_objective_update_best_genome() {
        let mut obj = ResearchObjective::new("test");
        let mut g1 = MutationStrategy::random(2, 1);
        g1.fitness = Some(0.5);
        obj.update_best_genome(g1);
        assert!(obj.best_genome.is_some());
        assert!((obj.best_genome.as_ref().unwrap().fitness.unwrap() - 0.5).abs() < 1e-10);

        // Better (lower) fitness replaces
        let mut g2 = MutationStrategy::random(2, 1);
        g2.fitness = Some(0.3);
        obj.update_best_genome(g2);
        assert!((obj.best_genome.as_ref().unwrap().fitness.unwrap() - 0.3).abs() < 1e-10);

        // Worse fitness does not replace
        let mut g3 = MutationStrategy::random(2, 1);
        g3.fitness = Some(0.8);
        obj.update_best_genome(g3);
        assert!((obj.best_genome.as_ref().unwrap().fitness.unwrap() - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_research_objective_update_best_genome_none_fitness_ignored() {
        let mut obj = ResearchObjective::new("test");
        let g = MutationStrategy::random(2, 1); // fitness is None
        obj.update_best_genome(g);
        assert!(obj.best_genome.is_none());
    }

    #[test]
    fn test_research_objective_status_transitions() {
        let mut obj = ResearchObjective::new("test");
        assert_eq!(obj.status, ResearchStatus::Active);

        obj.mark_stalled();
        assert_eq!(obj.status, ResearchStatus::Stalled);

        obj.mark_escalated();
        assert_eq!(obj.status, ResearchStatus::Escalated);

        obj.mark_completed();
        assert_eq!(obj.status, ResearchStatus::Completed);
    }
}
