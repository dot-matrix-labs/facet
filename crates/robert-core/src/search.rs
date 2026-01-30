use crate::llm::LlmClient;
use anyhow::Result;
use robert_graph::ingest::IngestionPipeline;
use robert_graph::query::GraphQuery;
use robert_graph::{GraphError, GraphStore, Node, VectorStore};
use std::sync::Arc;

pub struct SearchManager<S: GraphStore + VectorStore> {
    query_engine: GraphQuery<S>,
    ingestion_pipeline: Arc<IngestionPipeline<S>>,
    llm_client: Arc<LlmClient>,
}

impl<S: GraphStore + VectorStore + Clone> SearchManager<S> {
    pub fn new(
        store: S,
        ingestion_pipeline: Arc<IngestionPipeline<S>>,
        llm_client: Arc<LlmClient>,
    ) -> Self {
        Self {
            query_engine: GraphQuery::new(store),
            ingestion_pipeline,
            llm_client,
        }
    }

    pub async fn search(&self, query_text: &str, limit: usize) -> Result<Vec<Node>, GraphError> {
        // 1. Embed query
        let vector = self.ingestion_pipeline.embed_text(query_text).await?;

        // 2. Graph Search
        self.query_engine.search(vector, limit).await
    }

    pub async fn ask(&self, query_text: &str) -> Result<String> {
        // 1. Retrieve Context
        let nodes = self
            .search(query_text, 5)
            .await
            .map_err(|e| anyhow::anyhow!("Search failed: {}", e))?;

        // 2. Assemble Context
        let context_str = nodes
            .iter()
            .map(|n| {
                let content = n
                    .properties
                    .get("content_preview")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                format!("- [{}]: {}", n.label, content)
            })
            .collect::<Vec<_>>()
            .join("\n");

        // 3. Construct Prompt
        let system_prompt = "You are Robert, a helpful AI assistant with access to the user's personal documents. \
        Answer the user's question based ONLY on the provided context. If the context doesn't contain the answer, say so.";

        let user_prompt = format!("Context:\n{}\n\nQuestion: {}", context_str, query_text);

        // 4. Generate Answer
        self.llm_client
            .complete(&user_prompt, Some(system_prompt))
            .await
    }
}
