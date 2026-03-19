//! Life-graph analysis via Graph Neural Networks.
//!
//! When the `ruvnet-phase3` feature is enabled, delegates to `ruvector-gnn`
//! for hardware-accelerated GNN message passing. Otherwise provides a simple
//! neighborhood-aggregation fallback.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum GnnError {
    #[error("graph is empty")]
    EmptyGraph,
    #[error("node not found: {0}")]
    NodeNotFound(Uuid),
    #[error("gnn inference failed: {0}")]
    InferenceFailed(String),
}

pub type Result<T> = std::result::Result<T, GnnError>;

/// A node in the user's life graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: Uuid,
    pub label: String,
    pub domain: String,
    pub features: Vec<f32>,
}

/// A directed edge in the life graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: Uuid,
    pub target: Uuid,
    pub relation: String,
    pub weight: f32,
}

/// The full life graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LifeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl LifeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, label: &str, domain: &str, features: Vec<f32>) -> Uuid {
        let id = Uuid::new_v4();
        self.nodes.push(GraphNode {
            id,
            label: label.to_string(),
            domain: domain.to_string(),
            features,
        });
        id
    }

    pub fn add_edge(&mut self, source: Uuid, target: Uuid, relation: &str, weight: f32) {
        self.edges.push(GraphEdge {
            source,
            target,
            relation: relation.to_string(),
            weight,
        });
    }

    pub fn node(&self, id: Uuid) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn neighbors(&self, id: Uuid) -> Vec<(&GraphNode, &GraphEdge)> {
        self.edges
            .iter()
            .filter(|e| e.source == id)
            .filter_map(|e| self.node(e.target).map(|n| (n, e)))
            .collect()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

/// A predicted pattern from GNN analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternPrediction {
    pub node_id: Uuid,
    pub label: String,
    pub domain: String,
    pub score: f64,
    pub explanation: String,
}

/// Analyzes a user's life graph to find patterns and predictions.
#[derive(Debug, Clone)]
pub struct LifeGraphGnn {
    pub num_layers: usize,
    pub hidden_dim: usize,
}

impl LifeGraphGnn {
    pub fn new(num_layers: usize, hidden_dim: usize) -> Self {
        Self {
            num_layers,
            hidden_dim,
        }
    }

    /// Run inference on the life graph.
    pub fn predict(&self, graph: &LifeGraph) -> Result<Vec<PatternPrediction>> {
        if graph.nodes.is_empty() {
            return Err(GnnError::EmptyGraph);
        }

        #[cfg(feature = "ruvnet-phase3")]
        {
            self.predict_gnn(graph)
        }

        #[cfg(not(feature = "ruvnet-phase3"))]
        {
            self.predict_heuristic(graph)
        }
    }

    #[cfg(not(feature = "ruvnet-phase3"))]
    fn predict_heuristic(&self, graph: &LifeGraph) -> Result<Vec<PatternPrediction>> {
        let mut predictions = Vec::with_capacity(graph.nodes.len());

        for node in &graph.nodes {
            let neighbors = graph.neighbors(node.id);
            let feat_len = node.features.len();
            let mut agg = vec![0.0f64; feat_len];
            let mut total_weight = 0.0f64;

            for (neighbor, edge) in &neighbors {
                let w = edge.weight as f64;
                total_weight += w;
                for (i, &f) in neighbor.features.iter().enumerate() {
                    if i < feat_len {
                        agg[i] += f as f64 * w;
                    }
                }
            }

            if total_weight > 0.0 {
                for v in &mut agg {
                    *v /= total_weight;
                }
            }

            let dot: f64 = node
                .features
                .iter()
                .zip(agg.iter())
                .map(|(&a, &b)| a as f64 * b)
                .sum();
            let node_norm: f64 = node
                .features
                .iter()
                .map(|&v| (v as f64).powi(2))
                .sum::<f64>()
                .sqrt();
            let agg_norm: f64 = agg.iter().map(|v| v.powi(2)).sum::<f64>().sqrt();

            let score = if node_norm > 0.0 && agg_norm > 0.0 {
                (dot / (node_norm * agg_norm)).clamp(0.0, 1.0)
            } else if neighbors.is_empty() {
                0.5
            } else {
                0.0
            };

            let explanation = if neighbors.is_empty() {
                format!("'{}' is isolated -- no graph neighbors", node.label)
            } else {
                format!(
                    "'{}' has {} connections, alignment score {:.2}",
                    node.label,
                    neighbors.len(),
                    score
                )
            };

            predictions.push(PatternPrediction {
                node_id: node.id,
                label: node.label.clone(),
                domain: node.domain.clone(),
                score,
                explanation,
            });
        }

        Ok(predictions)
    }

    #[cfg(feature = "ruvnet-phase3")]
    fn predict_gnn(&self, graph: &LifeGraph) -> Result<Vec<PatternPrediction>> {
        use ruvector_gnn::{GnnModel, GraphData};

        let node_features: Vec<Vec<f32>> = graph.nodes.iter().map(|n| n.features.clone()).collect();
        let id_to_idx: std::collections::HashMap<Uuid, usize> = graph
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id, i))
            .collect();
        let edge_index: Vec<(usize, usize)> = graph
            .edges
            .iter()
            .filter_map(|e| {
                let s = id_to_idx.get(&e.source)?;
                let t = id_to_idx.get(&e.target)?;
                Some((*s, *t))
            })
            .collect();
        let edge_weights: Vec<f32> = graph.edges.iter().map(|e| e.weight).collect();

        let data = GraphData {
            node_features,
            edge_index,
            edge_weights,
        };
        let model = GnnModel::new(self.num_layers, self.hidden_dim);
        let scores = model
            .forward(&data)
            .map_err(|e| GnnError::InferenceFailed(format!("{e}")))?;

        let predictions = graph
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let score = scores.get(i).copied().unwrap_or(0.0) as f64;
                PatternPrediction {
                    node_id: node.id,
                    label: node.label.clone(),
                    domain: node.domain.clone(),
                    score,
                    explanation: format!(
                        "'{}' GNN score {:.3} (ruvector-gnn, {} layers)",
                        node.label, score, self.num_layers
                    ),
                }
            })
            .collect();

        Ok(predictions)
    }
}

impl Default for LifeGraphGnn {
    fn default() -> Self {
        Self::new(3, 32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_graph() -> LifeGraph {
        let mut g = LifeGraph::new();
        let groceries = g.add_node("groceries", "finance", vec![1.0, 0.0, 0.5]);
        let restaurants = g.add_node("restaurants", "finance", vec![0.8, 0.2, 0.6]);
        let health = g.add_node("gym", "health", vec![0.0, 1.0, 0.3]);
        g.add_edge(groceries, restaurants, "co-spend", 0.8);
        g.add_edge(groceries, health, "influences", 0.4);
        g.add_edge(restaurants, health, "correlates", 0.3);
        g
    }

    #[test]
    fn test_predict_returns_all_nodes() {
        let graph = sample_graph();
        let gnn = LifeGraphGnn::default();
        let predictions = gnn.predict(&graph).unwrap();
        assert_eq!(predictions.len(), 3);
    }

    #[test]
    fn test_empty_graph_error() {
        let graph = LifeGraph::new();
        let gnn = LifeGraphGnn::default();
        let err = gnn.predict(&graph).unwrap_err();
        assert!(matches!(err, GnnError::EmptyGraph));
    }

    #[test]
    fn test_isolated_node_gets_neutral_score() {
        let mut graph = LifeGraph::new();
        graph.add_node("isolated", "misc", vec![1.0, 0.5]);
        let gnn = LifeGraphGnn::default();
        let predictions = gnn.predict(&graph).unwrap();
        assert_eq!(predictions.len(), 1);
        assert!((predictions[0].score - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_connected_nodes_score() {
        let graph = sample_graph();
        let gnn = LifeGraphGnn::default();
        let predictions = gnn.predict(&graph).unwrap();
        let groceries = predictions.iter().find(|p| p.label == "groceries").unwrap();
        assert!(groceries.score >= 0.0 && groceries.score <= 1.0);
    }

    #[test]
    fn test_graph_construction() {
        let mut graph = LifeGraph::new();
        let a = graph.add_node("a", "test", vec![1.0]);
        let b = graph.add_node("b", "test", vec![0.5]);
        graph.add_edge(a, b, "relates", 1.0);
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        assert!(graph.node(a).is_some());
        assert_eq!(graph.neighbors(a).len(), 1);
        assert_eq!(graph.neighbors(b).len(), 0);
    }
}
