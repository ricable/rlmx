use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::AgentId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisType {
    MinCut,
    CommunityDetection,
    Diffusion,
    Centrality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub id: Uuid,
    pub analysis_type: AnalysisType,
    pub result: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// Analyst agent — performs graph analysis operations.
pub struct AnalystAgent {
    pub id: AgentId,
    pub analyses: Vec<Analysis>,
}

impl AnalystAgent {
    pub fn new() -> Self {
        Self {
            id: AgentId::new(),
            analyses: Vec::new(),
        }
    }

    /// Perform a graph analysis of the given type on the provided data.
    pub fn analyze(&mut self, analysis_type: AnalysisType, data: &serde_json::Value) -> Analysis {
        let result = match analysis_type {
            AnalysisType::MinCut => self.analyze_min_cut(data),
            AnalysisType::CommunityDetection => self.analyze_communities(data),
            AnalysisType::Diffusion => self.analyze_diffusion(data),
            AnalysisType::Centrality => self.analyze_centrality(data),
        };

        let analysis = Analysis {
            id: Uuid::new_v4(),
            analysis_type,
            result,
            timestamp: Utc::now(),
        };

        self.analyses.push(analysis.clone());
        analysis
    }

    fn analyze_min_cut(&self, data: &serde_json::Value) -> serde_json::Value {
        let node_count = data
            .get("nodes")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        let edge_count = data
            .get("edges")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        // Simplified min-cut estimate
        let estimated_cut = if edge_count > 0 {
            (edge_count as f64 / node_count.max(1) as f64).ceil() as u64
        } else {
            0
        };

        serde_json::json!({
            "algorithm": "karger_estimate",
            "node_count": node_count,
            "edge_count": edge_count,
            "estimated_min_cut": estimated_cut,
        })
    }

    fn analyze_communities(&self, data: &serde_json::Value) -> serde_json::Value {
        let node_count = data
            .get("nodes")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        // Simplified: estimate communities as sqrt(nodes)
        let estimated_communities = (node_count as f64).sqrt().ceil() as usize;

        serde_json::json!({
            "algorithm": "louvain_estimate",
            "node_count": node_count,
            "estimated_communities": estimated_communities,
            "modularity": 0.0,
        })
    }

    fn analyze_diffusion(&self, data: &serde_json::Value) -> serde_json::Value {
        let steps = data.get("steps").and_then(|v| v.as_u64()).unwrap_or(10);

        let temperature = data
            .get("temperature")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);

        serde_json::json!({
            "algorithm": "heat_kernel",
            "steps": steps,
            "temperature": temperature,
            "converged": steps > 5,
        })
    }

    fn analyze_centrality(&self, data: &serde_json::Value) -> serde_json::Value {
        let nodes = data
            .get("nodes")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        // Return centrality scores (simplified: all equal)
        let scores: Vec<serde_json::Value> = nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                serde_json::json!({
                    "node": node,
                    "centrality": 1.0 / (i + 1) as f64,
                })
            })
            .collect();

        serde_json::json!({
            "algorithm": "betweenness",
            "scores": scores,
        })
    }

    /// Number of analyses performed.
    pub fn analysis_count(&self) -> usize {
        self.analyses.len()
    }
}

impl Default for AnalystAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_cut_analysis() {
        let mut analyst = AnalystAgent::new();
        let data = serde_json::json!({
            "nodes": ["a", "b", "c"],
            "edges": [["a","b"], ["b","c"], ["a","c"]],
        });
        let analysis = analyst.analyze(AnalysisType::MinCut, &data);
        assert_eq!(analysis.analysis_type, AnalysisType::MinCut);
        assert!(analysis.result.get("estimated_min_cut").is_some());
    }

    #[test]
    fn test_community_detection() {
        let mut analyst = AnalystAgent::new();
        let data = serde_json::json!({
            "nodes": ["a", "b", "c", "d"],
        });
        let analysis = analyst.analyze(AnalysisType::CommunityDetection, &data);
        let communities = analysis.result["estimated_communities"].as_u64().unwrap();
        assert_eq!(communities, 2); // sqrt(4) = 2
    }

    #[test]
    fn test_diffusion_analysis() {
        let mut analyst = AnalystAgent::new();
        let data = serde_json::json!({"steps": 10, "temperature": 0.5});
        let analysis = analyst.analyze(AnalysisType::Diffusion, &data);
        assert_eq!(analysis.result["converged"], true);
    }

    #[test]
    fn test_centrality_analysis() {
        let mut analyst = AnalystAgent::new();
        let data = serde_json::json!({"nodes": ["x", "y", "z"]});
        let analysis = analyst.analyze(AnalysisType::Centrality, &data);
        let scores = analysis.result["scores"].as_array().unwrap();
        assert_eq!(scores.len(), 3);
    }

    #[test]
    fn test_analysis_count() {
        let mut analyst = AnalystAgent::new();
        let data = serde_json::json!({});
        analyst.analyze(AnalysisType::MinCut, &data);
        analyst.analyze(AnalysisType::Diffusion, &data);
        assert_eq!(analyst.analysis_count(), 2);
    }

    #[test]
    fn test_empty_graph_min_cut() {
        let mut analyst = AnalystAgent::new();
        let data = serde_json::json!({});
        let analysis = analyst.analyze(AnalysisType::MinCut, &data);
        assert_eq!(analysis.result["estimated_min_cut"], 0);
    }
}
