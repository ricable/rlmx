use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::{KernelError, KernelResult, MinCutAlgorithm};

/// A node in the kernel graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub node_type: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// An edge in the kernel graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub source: Uuid,
    pub target: Uuid,
    pub edge_type: String,
    pub weight: f64,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Simple in-memory graph supporting nodes, edges, basic Cypher-like queries,
/// min-cut, and heat kernel diffusion.
pub struct Graph {
    pub id: Uuid,
    nodes: HashMap<Uuid, Node>,
    edges: Vec<Edge>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Insert a node and return its id.
    pub fn insert_node(&mut self, node_type: impl Into<String>) -> Uuid {
        let id = Uuid::new_v4();
        let node = Node {
            id,
            node_type: node_type.into(),
            properties: HashMap::new(),
        };
        self.nodes.insert(id, node);
        id
    }

    /// Insert a node with properties.
    pub fn insert_node_with_props(
        &mut self,
        node_type: impl Into<String>,
        properties: HashMap<String, serde_json::Value>,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let node = Node {
            id,
            node_type: node_type.into(),
            properties,
        };
        self.nodes.insert(id, node);
        id
    }

    /// Insert an edge between two existing nodes.
    pub fn insert_edge(
        &mut self,
        source: Uuid,
        target: Uuid,
        edge_type: impl Into<String>,
        weight: f64,
    ) -> KernelResult<()> {
        if !self.nodes.contains_key(&source) {
            return Err(KernelError::GraphError(format!(
                "source node {} not found",
                source
            )));
        }
        if !self.nodes.contains_key(&target) {
            return Err(KernelError::GraphError(format!(
                "target node {} not found",
                target
            )));
        }
        self.edges.push(Edge {
            source,
            target,
            edge_type: edge_type.into(),
            weight,
            properties: HashMap::new(),
        });
        Ok(())
    }

    /// Get a node by id.
    pub fn get_node(&self, id: &Uuid) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    // -----------------------------------------------------------------------
    // Basic Cypher-like query
    // -----------------------------------------------------------------------

    /// Execute a basic Cypher-like query.
    ///
    /// Supports patterns of the form:
    ///   MATCH (n:Type)-[r:REL]->(m:Type) RETURN n,m
    ///
    /// Returns matching rows as JSON values.
    pub fn cypher_query(&self, query: &str) -> KernelResult<Vec<serde_json::Value>> {
        let parsed = CypherPattern::parse(query)?;
        let mut results = Vec::new();

        for edge in &self.edges {
            let src = match self.nodes.get(&edge.source) {
                Some(n) => n,
                None => continue,
            };
            let tgt = match self.nodes.get(&edge.target) {
                Some(n) => n,
                None => continue,
            };

            let src_match = parsed
                .source_type
                .as_ref()
                .map_or(true, |t| src.node_type == *t);
            let tgt_match = parsed
                .target_type
                .as_ref()
                .map_or(true, |t| tgt.node_type == *t);
            let rel_match = parsed
                .rel_type
                .as_ref()
                .map_or(true, |t| edge.edge_type == *t);

            if src_match && tgt_match && rel_match {
                let row = serde_json::json!({
                    &parsed.source_var: {
                        "id": src.id.to_string(),
                        "type": &src.node_type,
                        "properties": &src.properties,
                    },
                    &parsed.target_var: {
                        "id": tgt.id.to_string(),
                        "type": &tgt.node_type,
                        "properties": &tgt.properties,
                    },
                });
                results.push(row);
            }
        }

        Ok(results)
    }

    // -----------------------------------------------------------------------
    // Min-cut (Karger's randomised algorithm)
    // -----------------------------------------------------------------------

    /// Compute a min-cut using the specified algorithm.
    /// Returns (cut_weight, [partition_a_ids, partition_b_ids]).
    pub fn min_cut(&self, algorithm: &MinCutAlgorithm) -> KernelResult<(f64, Vec<Vec<Uuid>>)> {
        match algorithm {
            MinCutAlgorithm::Karger => self.karger_min_cut(),
            MinCutAlgorithm::StoerWagner => self.stoer_wagner_min_cut(),
        }
    }

    fn karger_min_cut(&self) -> KernelResult<(f64, Vec<Vec<Uuid>>)> {
        if self.nodes.len() < 2 {
            return Err(KernelError::GraphError(
                "need at least 2 nodes for min-cut".into(),
            ));
        }

        let node_ids: Vec<Uuid> = self.nodes.keys().copied().collect();
        let mut rng = rand::thread_rng();

        let mut best_cut = f64::MAX;
        let mut best_partitions = vec![vec![], vec![]];

        // Run Karger's algorithm multiple times for better results.
        let iterations = (self.nodes.len() * self.nodes.len()).max(10);

        for _ in 0..iterations {
            // Each node starts in its own supernode.
            let mut parent: HashMap<Uuid, Uuid> = node_ids.iter().map(|&id| (id, id)).collect();
            let mut sizes: HashMap<Uuid, Vec<Uuid>> =
                node_ids.iter().map(|&id| (id, vec![id])).collect();
            let mut num_supernodes = node_ids.len();

            while num_supernodes > 2 {
                // Pick a random edge.
                if self.edges.is_empty() {
                    break;
                }
                let idx = rng.gen_range(0..self.edges.len());
                let edge = &self.edges[idx];

                let root_s = find_root(&parent, edge.source);
                let root_t = find_root(&parent, edge.target);

                if root_s == root_t {
                    continue;
                }

                // Merge root_t into root_s.
                let members_t = sizes.remove(&root_t).unwrap_or_default();
                for m in &members_t {
                    parent.insert(*m, root_s);
                }
                sizes.entry(root_s).or_default().extend(members_t);
                num_supernodes -= 1;
            }

            // Count cut weight.
            let mut cut_weight = 0.0;
            for edge in &self.edges {
                let rs = find_root(&parent, edge.source);
                let rt = find_root(&parent, edge.target);
                if rs != rt {
                    cut_weight += edge.weight;
                }
            }

            if cut_weight < best_cut {
                best_cut = cut_weight;
                let parts: Vec<Vec<Uuid>> = sizes.values().cloned().collect();
                best_partitions = parts;
            }
        }

        Ok((best_cut, best_partitions))
    }

    fn stoer_wagner_min_cut(&self) -> KernelResult<(f64, Vec<Vec<Uuid>>)> {
        // Fallback to Karger for now.
        self.karger_min_cut()
    }

    // -----------------------------------------------------------------------
    // Heat kernel diffusion
    // -----------------------------------------------------------------------

    /// Approximate heat kernel diffusion: e^{-tL} * signal
    /// where L is the graph Laplacian and t = steps (treated as time parameter).
    ///
    /// Uses a simple Taylor expansion approximation:
    /// e^{-tL} ~ I - tL + (tL)^2/2! - ...
    pub fn diffuse(&self, signal: &[f64], steps: usize) -> KernelResult<Vec<f64>> {
        let n = self.nodes.len();
        if signal.len() != n {
            return Err(KernelError::GraphError(format!(
                "signal length {} != node count {}",
                signal.len(),
                n
            )));
        }

        let node_ids: Vec<Uuid> = self.nodes.keys().copied().collect();
        let id_to_idx: HashMap<Uuid, usize> =
            node_ids.iter().enumerate().map(|(i, &id)| (id, i)).collect();

        // Build adjacency matrix.
        let mut adj = vec![vec![0.0_f64; n]; n];
        for edge in &self.edges {
            if let (Some(&i), Some(&j)) = (id_to_idx.get(&edge.source), id_to_idx.get(&edge.target))
            {
                adj[i][j] += edge.weight;
                adj[j][i] += edge.weight;
            }
        }

        // Build Laplacian: L = D - A
        let mut laplacian = vec![vec![0.0_f64; n]; n];
        for i in 0..n {
            let degree: f64 = adj[i].iter().sum();
            laplacian[i][i] = degree;
            for j in 0..n {
                laplacian[i][j] -= adj[i][j];
            }
        }

        // Taylor expansion: result = (I - tL + (tL)^2/2! - ...) * signal
        let t = steps as f64;
        let terms = 6; // Number of Taylor terms.
        let mut result = signal.to_vec();

        // tL^k / k! applied iteratively.
        let mut current = signal.to_vec();
        for k in 1..=terms {
            let prev = current.clone();
            current = mat_vec_mul(&laplacian, &prev);
            // Scale by -t/k
            let scale = -t / (k as f64);
            for val in &mut current {
                *val *= scale;
            }
            for i in 0..n {
                result[i] += current[i];
            }
        }

        Ok(result)
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

fn find_root(parent: &HashMap<Uuid, Uuid>, mut node: Uuid) -> Uuid {
    while parent.get(&node).copied().unwrap_or(node) != node {
        node = parent[&node];
    }
    node
}

fn mat_vec_mul(mat: &[Vec<f64>], vec: &[f64]) -> Vec<f64> {
    let n = vec.len();
    let mut result = vec![0.0; n];
    for i in 0..n {
        for j in 0..n {
            result[i] += mat[i][j] * vec[j];
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Basic Cypher pattern parser
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct CypherPattern {
    source_var: String,
    source_type: Option<String>,
    target_var: String,
    target_type: Option<String>,
    rel_type: Option<String>,
}

impl CypherPattern {
    /// Parse a very basic Cypher pattern:
    /// MATCH (n:Type)-[r:REL]->(m:Type) RETURN n,m
    fn parse(query: &str) -> KernelResult<Self> {
        let query = query.trim();

        // Extract the MATCH ... RETURN part.
        let upper = query.to_uppercase();
        let match_start = upper
            .find("MATCH")
            .ok_or_else(|| KernelError::ParseError("missing MATCH keyword".into()))?;
        let return_start = upper
            .find("RETURN")
            .ok_or_else(|| KernelError::ParseError("missing RETURN keyword".into()))?;

        let pattern = &query[match_start + 5..return_start].trim();

        // Find source node: (var:Type) or (var)
        let src_start = pattern
            .find('(')
            .ok_or_else(|| KernelError::ParseError("missing source node '('".into()))?;
        let src_end = pattern
            .find(')')
            .ok_or_else(|| KernelError::ParseError("missing source node ')'".into()))?;
        let src_inner = &pattern[src_start + 1..src_end];
        let (source_var, source_type) = parse_node_spec(src_inner);

        // Find relationship: -[r:REL]->
        let rel_start = pattern.find('[');
        let rel_end = pattern.find(']');
        let rel_type = match (rel_start, rel_end) {
            (Some(s), Some(e)) => {
                let rel_inner = &pattern[s + 1..e];
                parse_rel_spec(rel_inner)
            }
            _ => None,
        };

        // Find target node: the last (...) in the pattern.
        let rest = &pattern[src_end + 1..];
        let tgt_start = rest
            .rfind('(')
            .ok_or_else(|| KernelError::ParseError("missing target node '('".into()))?;
        let tgt_end = rest
            .rfind(')')
            .ok_or_else(|| KernelError::ParseError("missing target node ')'".into()))?;
        let tgt_inner = &rest[tgt_start + 1..tgt_end];
        let (target_var, target_type) = parse_node_spec(tgt_inner);

        Ok(CypherPattern {
            source_var,
            source_type,
            target_var,
            target_type,
            rel_type,
        })
    }
}

fn parse_node_spec(spec: &str) -> (String, Option<String>) {
    let parts: Vec<&str> = spec.splitn(2, ':').collect();
    let var = parts[0].trim().to_string();
    let typ = parts.get(1).map(|t| t.trim().to_string());
    (
        if var.is_empty() {
            "_".to_string()
        } else {
            var
        },
        typ,
    )
}

fn parse_rel_spec(spec: &str) -> Option<String> {
    let parts: Vec<&str> = spec.splitn(2, ':').collect();
    parts.get(1).map(|t| t.trim().to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_node_and_edge() {
        let mut g = Graph::new();
        let a = g.insert_node("Person");
        let b = g.insert_node("Person");
        g.insert_edge(a, b, "KNOWS", 1.0).unwrap();

        assert_eq!(g.node_count(), 2);
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn test_insert_edge_missing_node_fails() {
        let mut g = Graph::new();
        let a = g.insert_node("Person");
        let fake = Uuid::new_v4();
        assert!(g.insert_edge(a, fake, "KNOWS", 1.0).is_err());
    }

    #[test]
    fn test_cypher_basic_query() {
        let mut g = Graph::new();
        let a = g.insert_node("Person");
        let b = g.insert_node("Company");
        let c = g.insert_node("Person");
        g.insert_edge(a, b, "WORKS_AT", 1.0).unwrap();
        g.insert_edge(c, b, "WORKS_AT", 1.0).unwrap();

        let results = g
            .cypher_query("MATCH (n:Person)-[r:WORKS_AT]->(m:Company) RETURN n,m")
            .unwrap();

        assert_eq!(results.len(), 2);
        for row in &results {
            assert_eq!(row["m"]["type"], "Company");
            assert_eq!(row["n"]["type"], "Person");
        }
    }
}
