use crate::{Edge, Node};
use petgraph::algo::kosaraju_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

pub struct EphemeralGraph {
    graph: DiGraph<Node, f32>,
    node_indices: HashMap<String, NodeIndex>,
}

impl Default for EphemeralGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl EphemeralGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
        }
    }

    pub fn from_nodes_and_edges(nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        let mut ephem = Self::new();

        for node in nodes {
            ephem.add_node(node);
        }

        for edge in edges {
            ephem.add_edge(edge);
        }

        ephem
    }

    pub fn add_node(&mut self, node: Node) {
        if !self.node_indices.contains_key(&node.id) {
            let id = node.id.clone();
            let idx = self.graph.add_node(node);
            self.node_indices.insert(id, idx);
        }
    }

    pub fn add_edge(&mut self, edge: Edge) {
        if let (Some(&source), Some(&target)) = (
            self.node_indices.get(&edge.source),
            self.node_indices.get(&edge.target),
        ) {
            self.graph.add_edge(source, target, edge.weight);
        }
    }

    pub fn get_strongly_connected_components(&self) -> Vec<Vec<Node>> {
        let scc = kosaraju_scc(&self.graph);
        scc.into_iter()
            .map(|indices| {
                indices
                    .into_iter()
                    .map(|idx| self.graph[idx].clone())
                    .collect()
            })
            .collect()
    }

    // Add more algorithms as needed (PageRank, Community Detection, etc.)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_node(id: &str) -> Node {
        Node {
            id: id.to_string(),
            label: "Test".to_string(),
            properties: json!({}),
            partition_id: "default".to_string(),
        }
    }

    fn create_edge(source: &str, target: &str) -> Edge {
        Edge {
            source: source.to_string(),
            target: target.to_string(),
            relation: "LINKS".to_string(),
            weight: 1.0,
            partition_id: "default".to_string(),
        }
    }

    #[test]
    fn test_ephemeral_graph_construction() {
        let mut graph = EphemeralGraph::new();
        let node1 = create_node("1");
        let node2 = create_node("2");

        graph.add_node(node1.clone());
        graph.add_node(node2.clone());
        graph.add_edge(create_edge("1", "2"));

        assert_eq!(graph.graph.node_count(), 2);
        assert_eq!(graph.graph.edge_count(), 1);
    }

    #[test]
    fn test_scc() {
        // Create a cycle: 1 -> 2 -> 3 -> 1
        let nodes = vec![create_node("1"), create_node("2"), create_node("3")];
        let edges = vec![
            create_edge("1", "2"),
            create_edge("2", "3"),
            create_edge("3", "1"),
        ];

        let graph = EphemeralGraph::from_nodes_and_edges(nodes, edges);
        let scc = graph.get_strongly_connected_components();

        assert_eq!(scc.len(), 1); // One component containing all 3 nodes
        assert_eq!(scc[0].len(), 3);
    }

    #[test]
    fn test_disconnected_components() {
        // 1 -> 2, 3 -> 4
        let nodes = vec![
            create_node("1"),
            create_node("2"),
            create_node("3"),
            create_node("4"),
        ];
        let edges = vec![create_edge("1", "2"), create_edge("3", "4")];

        let graph = EphemeralGraph::from_nodes_and_edges(nodes, edges);
        let scc = graph.get_strongly_connected_components();

        // Each node is its own SCC because there are no cycles, but kosaraju returns components.
        // Wait, Kosaraju returns strongly connected components. In a DAG (like 1->2), each node is an SCC.
        // So we expect 4 components.
        assert_eq!(scc.len(), 4);
    }

    #[test]
    fn test_duplicate_nodes() {
        let mut graph = EphemeralGraph::new();
        let node1 = create_node("1");

        graph.add_node(node1.clone());
        graph.add_node(node1.clone()); // Should be ignored

        assert_eq!(graph.graph.node_count(), 1);
    }
}
