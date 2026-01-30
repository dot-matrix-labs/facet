use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod ephemeral_graph;
pub mod ingest;
pub mod query;
pub mod surreal_store;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Node not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub properties: serde_json::Value,
    #[serde(default = "default_partition")]
    pub partition_id: String, // "personal", "work", "business", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub weight: f32,
    #[serde(default = "default_partition")]
    pub partition_id: String, // Same as nodes - edges belong to partitions
}

fn default_partition() -> String {
    "personal".to_string()
}

#[async_trait]
pub trait GraphStore: Send + Sync {
    async fn add_node(&self, node: Node) -> Result<(), GraphError>;
    async fn add_edge(&self, edge: Edge) -> Result<(), GraphError>;
    async fn get_node(&self, id: &str) -> Result<Node, GraphError>;
    async fn get_neighbors(&self, id: &str) -> Result<Vec<(Edge, Node)>, GraphError>;
    async fn update_node(&self, node: Node) -> Result<(), GraphError>;

    // Partition-aware queries
    async fn query_by_partition(&self, partition_id: &str) -> Result<Vec<Node>, GraphError>;
    async fn get_neighbors_in_partition(
        &self,
        id: &str,
        partition_id: &str,
    ) -> Result<Vec<(Edge, Node)>, GraphError>;
}

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn add_embedding(&self, id: &str, vector: Vec<f32>) -> Result<(), GraphError>;
    async fn search(
        &self,
        vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<(String, f32)>, GraphError>;
}

#[cfg(any(test, feature = "test-utils"))]
pub mod mocks {
    use super::*;

    // Mock implementation for TDD
    pub struct MockGraphStore {
        nodes: std::sync::RwLock<std::collections::HashMap<String, Node>>,
        edges: std::sync::RwLock<Vec<Edge>>,
    }

    impl Default for MockGraphStore {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockGraphStore {
        pub fn new() -> Self {
            Self {
                nodes: std::sync::RwLock::new(std::collections::HashMap::new()),
                edges: std::sync::RwLock::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl GraphStore for MockGraphStore {
        async fn add_node(&self, node: Node) -> Result<(), GraphError> {
            let mut nodes = self.nodes.write().unwrap();
            nodes.insert(node.id.clone(), node);
            Ok(())
        }

        async fn add_edge(&self, edge: Edge) -> Result<(), GraphError> {
            let mut edges = self.edges.write().unwrap();
            edges.push(edge);
            Ok(())
        }

        async fn get_node(&self, id: &str) -> Result<Node, GraphError> {
            let nodes = self.nodes.read().unwrap();
            nodes
                .get(id)
                .cloned()
                .ok_or(GraphError::NotFound(id.to_string()))
        }

        async fn get_neighbors(&self, id: &str) -> Result<Vec<(Edge, Node)>, GraphError> {
            let edges = self.edges.read().unwrap();
            let nodes = self.nodes.read().unwrap();

            let mut result = Vec::new();
            for edge in edges.iter() {
                if edge.source == id {
                    if let Some(target_node) = nodes.get(&edge.target) {
                        result.push((edge.clone(), target_node.clone()));
                    }
                }
            }
            Ok(result)
        }

        async fn update_node(&self, node: Node) -> Result<(), GraphError> {
            let mut nodes = self.nodes.write().unwrap();
            if !nodes.contains_key(&node.id) {
                return Err(GraphError::NotFound(node.id));
            }
            nodes.insert(node.id.clone(), node);
            Ok(())
        }

        async fn query_by_partition(&self, partition_id: &str) -> Result<Vec<Node>, GraphError> {
            let nodes = self.nodes.read().unwrap();
            let filtered: Vec<Node> = nodes
                .values()
                .filter(|n| n.partition_id == partition_id)
                .cloned()
                .collect();
            Ok(filtered)
        }

        async fn get_neighbors_in_partition(
            &self,
            id: &str,
            partition_id: &str,
        ) -> Result<Vec<(Edge, Node)>, GraphError> {
            let edges = self.edges.read().unwrap();
            let nodes = self.nodes.read().unwrap();

            let mut result = Vec::new();
            for edge in edges.iter() {
                if edge.source == id && edge.partition_id == partition_id {
                    if let Some(target_node) = nodes.get(&edge.target) {
                        if target_node.partition_id == partition_id {
                            result.push((edge.clone(), target_node.clone()));
                        }
                    }
                }
            }
            Ok(result)
        }
    }

    pub struct MockVectorStore {
        vectors: std::sync::RwLock<std::collections::HashMap<String, Vec<f32>>>,
    }

    impl Default for MockVectorStore {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockVectorStore {
        pub fn new() -> Self {
            Self {
                vectors: std::sync::RwLock::new(std::collections::HashMap::new()),
            }
        }

        fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
            let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
            let norm_a: f32 = v1.iter().map(|a| a * a).sum::<f32>().sqrt();
            let norm_b: f32 = v2.iter().map(|b| b * b).sum::<f32>().sqrt();
            if norm_a == 0.0 || norm_b == 0.0 {
                0.0
            } else {
                dot_product / (norm_a * norm_b)
            }
        }
    }

    #[async_trait]
    impl VectorStore for MockVectorStore {
        async fn add_embedding(&self, id: &str, vector: Vec<f32>) -> Result<(), GraphError> {
            let mut vectors = self.vectors.write().unwrap();
            vectors.insert(id.to_string(), vector);
            Ok(())
        }

        async fn search(
            &self,
            query: Vec<f32>,
            limit: usize,
        ) -> Result<Vec<(String, f32)>, GraphError> {
            let vectors = self.vectors.read().unwrap();
            let mut results: Vec<(String, f32)> = vectors
                .iter()
                .map(|(id, vec)| (id.clone(), Self::cosine_similarity(&query, vec)))
                .collect();

            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            results.truncate(limit);
            Ok(results)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::{MockGraphStore, MockVectorStore};

    #[tokio::test]
    async fn test_graph_operations() {
        let store = MockGraphStore::new();

        let node1 = Node {
            id: "1".to_string(),
            label: "Person".to_string(),
            properties: serde_json::json!({"name": "Alice"}),
            partition_id: "personal".to_string(),
        };

        let node2 = Node {
            id: "2".to_string(),
            label: "Person".to_string(),
            properties: serde_json::json!({"name": "Bob"}),
            partition_id: "personal".to_string(),
        };

        let edge = Edge {
            source: "1".to_string(),
            target: "2".to_string(),
            relation: "KNOWS".to_string(),
            weight: 1.0,
            partition_id: "personal".to_string(),
        };

        store.add_node(node1.clone()).await.unwrap();
        store.add_node(node2.clone()).await.unwrap();
        store.add_edge(edge.clone()).await.unwrap();

        let retrieved = store.get_node("1").await.unwrap();
        assert_eq!(retrieved, node1);

        let neighbors = store.get_neighbors("1").await.unwrap();
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].1, node2);
        assert_eq!(neighbors[0].0.relation, "KNOWS");
    }

    #[tokio::test]
    async fn test_vector_operations() {
        let store = MockVectorStore::new();

        // Simple 2D vectors for testing
        store.add_embedding("vec1", vec![1.0, 0.0]).await.unwrap();
        store.add_embedding("vec2", vec![0.0, 1.0]).await.unwrap();
        store
            .add_embedding("vec3", vec![0.707, 0.707])
            .await
            .unwrap(); // ~45 degrees

        let query = vec![1.0, 0.0];
        let results = store.search(query, 3).await.unwrap();

        assert_eq!(results[0].0, "vec1");
        assert!((results[0].1 - 1.0).abs() < 0.001); // Exact match

        assert_eq!(results[1].0, "vec3"); // Closer than vec2
        assert!((results[1].1 - 0.707).abs() < 0.001);
    }
}
