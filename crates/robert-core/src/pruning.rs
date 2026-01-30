use robert_graph::{GraphError, GraphStore};
use serde_json::json;

pub struct PruningManager<G: GraphStore> {
    graph_store: G,
}

impl<G: GraphStore> PruningManager<G> {
    pub fn new(graph_store: G) -> Self {
        Self { graph_store }
    }

    pub async fn mark_as_outdated(&self, node_id: &str) -> Result<(), GraphError> {
        let mut node = self.graph_store.get_node(node_id).await?;

        // Update properties
        if let Some(props) = node.properties.as_object_mut() {
            props.insert("status".to_string(), json!("outdated"));
            props.insert(
                "pruned_at".to_string(),
                json!(chrono::Utc::now().to_rfc3339()),
            );
        }

        self.graph_store.update_node(node).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use robert_graph::mocks::MockGraphStore;
    use robert_graph::Node;

    #[tokio::test]
    async fn test_pruning() {
        let graph = MockGraphStore::new();
        let manager = PruningManager::new(graph);

        // Setup
        let node = Node {
            id: "doc1".to_string(),
            label: "Document".to_string(),
            properties: json!({"status": "active"}),
        };
        manager.graph_store.add_node(node.clone()).await.unwrap();

        // Prune
        manager.mark_as_outdated("doc1").await.unwrap();

        // Verify
        let updated = manager.graph_store.get_node("doc1").await.unwrap();
        assert_eq!(updated.properties["status"], "outdated");
        assert!(updated.properties.get("pruned_at").is_some());
    }
}
