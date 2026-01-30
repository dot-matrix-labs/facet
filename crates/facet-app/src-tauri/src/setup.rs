use robert_core::context::ContextManager;
use robert_core::llm::LlmClient;
use robert_core::search::SearchManager;
use robert_graph::ingest::IngestionPipeline;
use robert_graph::surreal_store::SurrealStore;
use std::path::PathBuf;
use std::sync::Arc;

pub struct RobertState {
    pub context_manager: Arc<ContextManager>,
    pub search_manager: Arc<SearchManager<SurrealStore>>,
}

pub async fn init_backend() -> Result<RobertState, String> {
    // 1. Initialize Context Manager
    let context_manager = Arc::new(ContextManager::new());

    // 2. Initialize SurrealDB
    // Use a data directory in the user's home or app data
    let data_dir = dirs::home_dir()
        .unwrap_or(PathBuf::from("."))
        .join(".robert/data");
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let store = SurrealStore::new(data_dir)
        .await
        .map_err(|e| e.to_string())?;

    // 3. Initialize Ingestion Pipeline
    let ingestion_pipeline =
        Arc::new(IngestionPipeline::new(store.clone()).map_err(|e| e.to_string())?);

    // 4. Initialize LLM Client
    let llm_client = Arc::new(LlmClient::from_env());

    // 5. Initialize Search Manager
    let search_manager = Arc::new(SearchManager::new(store, ingestion_pipeline, llm_client));

    Ok(RobertState {
        context_manager,
        search_manager,
    })
}
