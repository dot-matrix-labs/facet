use crate::{GraphError, GraphStore, Node, VectorStore};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use uuid::Uuid;

pub struct IngestionPipeline<S: GraphStore + VectorStore> {
    store: S,
    embedding_model: TextEmbedding,
}

impl<S: GraphStore + VectorStore> IngestionPipeline<S> {
    pub fn new(store: S) -> Result<Self, GraphError> {
        let mut options = InitOptions::new(EmbeddingModel::AllMiniLML6V2);
        options.show_download_progress = true;
        let model = TextEmbedding::try_new(options)
            .map_err(|e| GraphError::Storage(format!("Failed to load embedding model: {}", e)))?;

        Ok(Self {
            store,
            embedding_model: model,
        })
    }

    pub async fn process_document(&self, title: &str, content: &str, partition_id: &str) -> Result<String, GraphError> {
        let doc_id = Uuid::new_v4().to_string();

        // 1. Create Document Node
        let node = Node {
            id: doc_id.clone(),
            label: "Document".to_string(),
            properties: serde_json::json!({
                "title": title,
                "content_preview": content.chars().take(100).collect::<String>(),
                "length": content.len()
            }),
            partition_id: partition_id.to_string(),
        };
        self.store.add_node(node).await?;

        // 2. Generate Embedding using FastEmbed
        let documents = vec![content.to_string()];
        let embeddings = self
            .embedding_model
            .embed(documents, None)
            .map_err(|e| GraphError::Storage(format!("Embedding failed: {}", e)))?;

        if let Some(embedding) = embeddings.first() {
            self.store.add_embedding(&doc_id, embedding.clone()).await?;
        }

        Ok(doc_id)
    }

    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, GraphError> {
        let documents = vec![text.to_string()];
        let embeddings = self
            .embedding_model
            .embed(documents, None)
            .map_err(|e| GraphError::Storage(format!("Embedding failed: {}", e)))?;

        embeddings
            .first()
            .cloned()
            .ok_or(GraphError::Storage("No embedding generated".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::{MockGraphStore, MockVectorStore};
    use crate::{Edge, Node};
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
    async fn test_ingestion_pipeline() {
        let store = MockStore::new();
        // Note: This test will try to download the model, which might be slow or fail in some envs.
        // For unit tests, we might want to mock the embedding model too, but FastEmbed struct is hard to mock.
        // We'll skip if model loading fails (e.g. no network) or just run it.

        let pipeline = IngestionPipeline::new(store);
        if let Ok(pipeline) = pipeline {
            let doc_id = pipeline
                .process_document("Test Doc", "This is some content.", "personal")
                .await
                .unwrap();

            // Verify Graph
            let node = pipeline.store.get_node(&doc_id).await.unwrap();
            assert_eq!(node.label, "Document");
        }
    }
}
