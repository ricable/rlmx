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

        let mut node_ids: Vec<Uuid> = self.nodes.keys().copied().collect();
        node_ids.sort();
        let mut rng = rand::thread_rng();

        let mut best_cut = f64::MAX;
        let mut best_partitions = vec![vec![], vec![]];

        // Run Karger's algorithm multiple times for better results.
        // Cap iterations to prevent DoS on large graphs.
        const MAX_ITERATIONS: usize = 10_000;
        let iterations = (self.nodes.len() * self.nodes.len()).max(10).min(MAX_ITERATIONS);

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

                let root_s = find_root(&mut parent, edge.source);
                let root_t = find_root(&mut parent, edge.target);

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
                let rs = find_root(&mut parent, edge.source);
                let rt = find_root(&mut parent, edge.target);
                if rs != rt {
                    cut_weight += edge.weight;
                }
            }

            if cut_weight < best_cut {
                best_cut = cut_weight;
                let parts: Vec<Vec<Uuid>> = sizes.values().filter(|v| !v.is_empty()).cloned().collect();
                best_partitions = parts;
            }
        }

        Ok((best_cut, best_partitions))
    }

    fn stoer_wagner_min_cut(&self) -> KernelResult<(f64, Vec<Vec<Uuid>>)> {
        if self.nodes.len() < 2 {
            return Err(KernelError::GraphError(
                "need at least 2 nodes for min-cut".into(),
            ));
        }

        // Build an adjacency matrix using merged-node indices.
        // Each "supernode" is a set of original node ids.
        // Sort for deterministic index assignment.
        let mut node_ids: Vec<Uuid> = self.nodes.keys().copied().collect();
        node_ids.sort();
        let n = node_ids.len();
        let id_to_idx: HashMap<Uuid, usize> =
            node_ids.iter().enumerate().map(|(i, &id)| (id, i)).collect();

        // Weighted adjacency matrix (symmetric, undirected).
        let mut w = vec![vec![0.0_f64; n]; n];
        for edge in &self.edges {
            if let (Some(&i), Some(&j)) =
                (id_to_idx.get(&edge.source), id_to_idx.get(&edge.target))
            {
                w[i][j] += edge.weight;
                w[j][i] += edge.weight;
            }
        }

        // Track which original nodes belong to each supernode.
        let mut groups: Vec<Vec<Uuid>> = node_ids.iter().map(|&id| vec![id]).collect();

        // active[i] is true if supernode i has not been merged away.
        let mut active = vec![true; n];

        let mut best_cut = f64::MAX;
        let mut best_partition: Vec<Vec<Uuid>> = vec![vec![], vec![]];

        // Stoer-Wagner runs n-1 phases.
        for _ in 0..n - 1 {
            // Maximum adjacency ordering among active nodes.
            let active_nodes: Vec<usize> = (0..n).filter(|&i| active[i]).collect();
            if active_nodes.len() < 2 {
                break;
            }

            let start = active_nodes[0];
            let mut in_a = vec![false; n];
            let mut key = vec![0.0_f64; n]; // connectivity to the growing set A

            let mut last = start;
            let mut second_last = start;

            for phase_step in 0..active_nodes.len() {
                // Pick the active node not yet in A with the largest key.
                let mut best_node = None;
                let mut best_key = -1.0_f64;
                for &v in &active_nodes {
                    if !in_a[v] {
                        if phase_step == 0 || key[v] > best_key {
                            best_key = key[v];
                            best_node = Some(v);
                        }
                    }
                }
                let v = match best_node {
                    Some(v) => v,
                    None => break, // no more candidates in this phase
                };
                in_a[v] = true;
                second_last = last;
                last = v;

                // Update keys for neighbours.
                for &u in &active_nodes {
                    if !in_a[u] {
                        key[u] += w[v][u];
                    }
                }
            }

            // The last node added is t, second-to-last is s.
            // The cut-of-the-phase is key[t] (sum of edges from t to the rest).
            let t = last;
            let s = second_last;
            let cut_of_phase = key[t];

            if cut_of_phase < best_cut {
                best_cut = cut_of_phase;
                // Partition: group(t) vs everything else.
                let t_group = groups[t].clone();
                let rest: Vec<Uuid> = (0..n)
                    .filter(|&i| active[i] && i != t)
                    .flat_map(|i| groups[i].iter().copied())
                    .collect();
                best_partition = vec![t_group, rest];
            }

            // Merge t into s: add t's edges to s, deactivate t.
            let t_members = std::mem::take(&mut groups[t]);
            groups[s].extend(t_members);
            active[t] = false;

            for i in 0..n {
                w[s][i] += w[t][i];
                w[i][s] += w[i][t];
            }
            w[s][s] = 0.0; // no self-loops
            // Zero out t's row/col to be safe.
            for i in 0..n {
                w[t][i] = 0.0;
                w[i][t] = 0.0;
            }
        }

        Ok((best_cut, best_partition))
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

        // Sort node IDs for deterministic position-to-node mapping.
        let mut node_ids: Vec<Uuid> = self.nodes.keys().copied().collect();
        node_ids.sort();
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

        // Approximate e^{-tL} * signal using iterated Taylor expansion.
        // Instead of e^{-steps*L} (diverges for large steps with few terms),
        // apply e^{-L} repeatedly `steps` times, each with a 10-term Taylor
        // expansion for t=1.0 which converges well.
        let t = 1.0_f64;
        let terms = 10;
        let mut result = signal.to_vec();

        for _step in 0..steps {
            let mut step_result = result.clone();
            let mut current_term = result.clone();
            for k in 1..=terms {
                current_term = mat_vec_mul(&laplacian, &current_term);
                let scale = -t / (k as f64);
                for val in &mut current_term {
                    *val *= scale;
                }
                for i in 0..n {
                    step_result[i] += current_term[i];
                }
            }
            result = step_result;
        }

        Ok(result)
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

fn find_root(parent: &mut HashMap<Uuid, Uuid>, mut node: Uuid) -> Uuid {
    // Path halving: every other node on the path points to its grandparent.
    loop {
        let p = parent.get(&node).copied().unwrap_or(node);
        if p == node {
            break;
        }
        let gp = parent.get(&p).copied().unwrap_or(p);
        parent.insert(node, gp);
        node = gp;
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
        let (source_var, source_type) = parse_node_spec(src_inner)?;

        // Find relationship: -[r:REL]->
        let rel_start = pattern.find('[');
        let rel_end = pattern.find(']');
        let rel_type = match (rel_start, rel_end) {
            (Some(s), Some(e)) => {
                let rel_inner = &pattern[s + 1..e];
                parse_rel_spec(rel_inner)?
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
        let (target_var, target_type) = parse_node_spec(tgt_inner)?;

        Ok(CypherPattern {
            source_var,
            source_type,
            target_var,
            target_type,
            rel_type,
        })
    }
}

/// Validate that an identifier contains only alphanumeric characters and underscores.
fn validate_identifier(s: &str) -> KernelResult<()> {
    if s.is_empty() {
        return Ok(());
    }
    if s.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Ok(())
    } else {
        Err(KernelError::ParseError(format!(
            "invalid identifier '{}': only alphanumeric and underscore allowed",
            s
        )))
    }
}

fn parse_node_spec(spec: &str) -> KernelResult<(String, Option<String>)> {
    let parts: Vec<&str> = spec.splitn(2, ':').collect();
    let var = parts[0].trim().to_string();
    let typ = parts.get(1).map(|t| t.trim().to_string());
    validate_identifier(&var)?;
    if let Some(ref t) = typ {
        validate_identifier(t)?;
    }
    Ok((
        if var.is_empty() {
            "_".to_string()
        } else {
            var
        },
        typ,
    ))
}

fn parse_rel_spec(spec: &str) -> KernelResult<Option<String>> {
    let parts: Vec<&str> = spec.splitn(2, ':').collect();
    // Validate the relationship variable (e.g. "r" in "r:KNOWS").
    let var = parts[0].trim();
    validate_identifier(var)?;
    let typ = parts.get(1).map(|t| t.trim().to_string());
    if let Some(ref t) = typ {
        validate_identifier(t)?;
    }
    Ok(typ)
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

    /// Build a 4-node graph: two pairs (a-b, c-d) with heavy internal edges
    /// and a single light edge between pairs. The min-cut should be the light edge.
    fn build_two_pair_graph() -> (Graph, Uuid, Uuid, Uuid, Uuid) {
        let mut g = Graph::new();
        let a = g.insert_node("N");
        let b = g.insert_node("N");
        let c = g.insert_node("N");
        let d = g.insert_node("N");
        // Heavy edges within pairs
        g.insert_edge(a, b, "HEAVY", 10.0).unwrap();
        g.insert_edge(c, d, "HEAVY", 10.0).unwrap();
        // Light edge between pairs
        g.insert_edge(b, c, "LIGHT", 1.0).unwrap();
        (g, a, b, c, d)
    }

    #[test]
    fn test_stoer_wagner_min_cut_simple() {
        let (g, a, b, c, d) = build_two_pair_graph();
        let (cut_weight, partitions) = g.min_cut(&MinCutAlgorithm::StoerWagner).unwrap();

        assert!(
            (cut_weight - 1.0).abs() < 1e-9,
            "expected cut weight 1.0, got {}",
            cut_weight
        );
        assert_eq!(partitions.len(), 2);
        assert!(!partitions[0].is_empty(), "partition 0 should be non-empty");
        assert!(!partitions[1].is_empty(), "partition 1 should be non-empty");

        // One partition should contain {a, b} and the other {c, d} (in any order).
        let mut p0: Vec<Uuid> = partitions[0].clone();
        let mut p1: Vec<Uuid> = partitions[1].clone();
        p0.sort();
        p1.sort();
        let mut pair_ab = vec![a, b];
        let mut pair_cd = vec![c, d];
        pair_ab.sort();
        pair_cd.sort();

        let correct = (p0 == pair_ab && p1 == pair_cd) || (p0 == pair_cd && p1 == pair_ab);
        assert!(correct, "partitions should be {{a,b}} and {{c,d}}");
    }

    #[test]
    fn test_karger_min_cut_simple() {
        let (g, _a, _b, _c, _d) = build_two_pair_graph();
        let (cut_weight, partitions) = g.min_cut(&MinCutAlgorithm::Karger).unwrap();

        // Karger is randomized; with heavy/light gap it should find the right cut.
        assert!(
            (cut_weight - 1.0).abs() < 1e-9,
            "expected cut weight 1.0, got {}",
            cut_weight
        );
        assert_eq!(partitions.len(), 2);
        assert!(!partitions[0].is_empty(), "partition 0 should be non-empty");
        assert!(!partitions[1].is_empty(), "partition 1 should be non-empty");
    }

    #[test]
    fn test_stoer_wagner_triangle() {
        // Triangle: 3 nodes, all edges weight 1. Min-cut = 2.
        let mut g = Graph::new();
        let a = g.insert_node("N");
        let b = g.insert_node("N");
        let c = g.insert_node("N");
        g.insert_edge(a, b, "E", 1.0).unwrap();
        g.insert_edge(b, c, "E", 1.0).unwrap();
        g.insert_edge(a, c, "E", 1.0).unwrap();

        let (cut_weight, partitions) = g.min_cut(&MinCutAlgorithm::StoerWagner).unwrap();

        assert!(
            (cut_weight - 2.0).abs() < 1e-9,
            "expected cut weight 2.0, got {}",
            cut_weight
        );
        assert_eq!(partitions.len(), 2);
        assert!(!partitions[0].is_empty());
        assert!(!partitions[1].is_empty());
    }

    #[test]
    fn test_min_cut_too_few_nodes() {
        let mut g = Graph::new();
        g.insert_node("N");
        assert!(g.min_cut(&MinCutAlgorithm::StoerWagner).is_err());
        assert!(g.min_cut(&MinCutAlgorithm::Karger).is_err());
    }

    #[test]
    fn test_diffuse_two_node_graph() {
        // 2-node graph with edge weight 1.0.
        // Laplacian: [[1, -1], [-1, 1]]
        // Signal [1.0, 0.0] should diffuse toward [0.5, 0.5].
        let mut g = Graph::new();
        let a = g.insert_node("A");
        let b = g.insert_node("B");
        g.insert_edge(a, b, "E", 1.0).unwrap();

        // Determine sorted order so we know which index is which.
        let mut ids = vec![a, b];
        ids.sort();
        let signal = if ids[0] == a {
            vec![1.0, 0.0]
        } else {
            vec![0.0, 1.0]
        };

        let result = g.diffuse(&signal, 1).unwrap();
        assert_eq!(result.len(), 2);

        // Signal should have moved toward equilibrium.
        let sum: f64 = result.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6, "Signal sum should be ~1.0, got {}", sum);

        // Both values should be between 0 and 1.
        for (i, &v) in result.iter().enumerate() {
            assert!(v >= 0.0 && v <= 1.0, "result[{}] = {} out of [0,1]", i, v);
        }

        // The originally-1.0 node should have decreased, the 0.0 node increased.
        let a_idx = if ids[0] == a { 0 } else { 1 };
        let b_idx = 1 - a_idx;
        assert!(result[a_idx] < 1.0, "Node A should have diffused away");
        assert!(result[b_idx] > 0.0, "Node B should have received signal");
    }

    #[test]
    fn test_diffuse_signal_length_mismatch() {
        let mut g = Graph::new();
        g.insert_node("A");
        g.insert_node("B");
        // Signal has wrong length.
        assert!(g.diffuse(&[1.0, 0.0, 0.0], 1).is_err());
    }

    #[test]
    fn test_cypher_rejects_invalid_identifiers() {
        let mut g = Graph::new();
        g.insert_node("Person");
        // Query with special characters in type should be rejected.
        let result = g.cypher_query("MATCH (n:Per;son)-[r:KNOWS]->(m) RETURN n,m");
        assert!(result.is_err());
    }
}
