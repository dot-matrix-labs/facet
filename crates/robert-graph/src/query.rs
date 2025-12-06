use crate::ephemeral_graph::EphemeralGraph;
use crate::{GraphError, GraphStore, Node, VectorStore};
use std::collections::HashSet;

pub struct GraphQuery<S: GraphStore + VectorStore> {
    store: S,
}

impl<S: GraphStore + VectorStore> GraphQuery<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<Node>, GraphError> {
        // 1. Vector Search to get entry points
        let initial_results = self.store.search(query_vector, limit).await?;

        let mut visited = HashSet::new();
        let mut subgraph_nodes = Vec::new();
        let mut subgraph_edges = Vec::new();

        // 2. Load Subgraph (BFS from entry points)
        // For Alpha, we'll do a simple 1-hop expansion from vector search results
        for (id, _score) in initial_results {
            if visited.contains(&id) {
                continue;
            }

            if let Ok(node) = self.store.get_node(&id).await {
                visited.insert(id.clone());
                subgraph_nodes.push(node.clone());

                // Get neighbors
                if let Ok(neighbors) = self.store.get_neighbors(&id).await {
                    for (edge, target_node) in neighbors {
                        subgraph_edges.push(edge);
                        if !visited.contains(&target_node.id) {
                            visited.insert(target_node.id.clone());
                            subgraph_nodes.push(target_node);
                        }
                    }
                }
            }
        }

        // 3. Build Ephemeral Graph for reasoning (e.g. finding paths, communities)
        let _graph = EphemeralGraph::from_nodes_and_edges(subgraph_nodes.clone(), subgraph_edges);

        // For now, just return the nodes found
        // In v1.0, we would run PageRank or Community Detection here
        Ok(subgraph_nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::{MockGraphStore, MockVectorStore};
    use crate::Edge;
    use async_trait::async_trait;

    // Combined mock for testing
    struct MockStore {
        graph: MockGraphStore,
        vector: MockVectorStore,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                graph: MockGraphStore::new(),
                vector: MockVectorStore::new(),
            }
        }
    }

    #[async_trait]
    impl GraphStore for MockStore {
        async fn add_node(&self, node: Node) -> Result<(), GraphError> {
            self.graph.add_node(node).await
        }
        async fn add_edge(&self, edge: Edge) -> Result<(), GraphError> {
            self.graph.add_edge(edge).await
        }
        async fn get_node(&self, id: &str) -> Result<Node, GraphError> {
            self.graph.get_node(id).await
        }
        async fn get_neighbors(&self, id: &str) -> Result<Vec<(Edge, Node)>, GraphError> {
            self.graph.get_neighbors(id).await
        }
        async fn update_node(&self, node: Node) -> Result<(), GraphError> {
            self.graph.update_node(node).await
        }

        async fn query_by_partition(&self, partition_id: &str) -> Result<Vec<Node>, GraphError> {
            self.graph.query_by_partition(partition_id).await
        }

        async fn get_neighbors_in_partition(
            &self,
            id: &str,
            partition_id: &str,
        ) -> Result<Vec<(Edge, Node)>, GraphError> {
            self.graph.get_neighbors_in_partition(id, partition_id).await
        }
    }

    #[async_trait]
    impl VectorStore for MockStore {
        async fn add_embedding(&self, id: &str, vector: Vec<f32>) -> Result<(), GraphError> {
            self.vector.add_embedding(id, vector).await
        }
        async fn search(
            &self,
            vector: Vec<f32>,
            limit: usize,
        ) -> Result<Vec<(String, f32)>, GraphError> {
            self.vector.search(vector, limit).await
        }
    }

    #[tokio::test]
    async fn test_graph_traversal() {
        let store = MockStore::new();

        // Setup data
        let node1 = Node {
            id: "1".to_string(),
            label: "Doc".to_string(),
            properties: serde_json::json!({}),
            partition_id: "default".to_string(),
        };
        let node2 = Node {
            id: "2".to_string(),
            label: "Doc".to_string(),
            properties: serde_json::json!({}),
            partition_id: "default".to_string(),
        };
        store.add_node(node1.clone()).await.unwrap();
        store.add_node(node2.clone()).await.unwrap();

        // Mock vector search result
        store.add_embedding("1", vec![1.0, 0.0]).await.unwrap();

        let query = GraphQuery::new(store);
        let results = query.search(vec![1.0, 0.0], 1).await.unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].id, "1");
    }
}
