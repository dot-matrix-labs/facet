use crate::{Edge, GraphError, GraphStore, Node, VectorStore};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;

#[derive(Clone)]
pub struct SurrealStore {
    db: Surreal<Db>,
}

impl SurrealStore {
    pub async fn new(path: PathBuf) -> Result<Self, GraphError> {
        let db = Surreal::new::<RocksDb>(path)
            .await
            .map_err(|e| GraphError::Storage(e.to_string()))?;

        db.use_ns("robert")
            .use_db("core")
            .await
            .map_err(|e| GraphError::Storage(e.to_string()))?;

        Ok(Self { db })
    }
}

#[async_trait]
impl GraphStore for SurrealStore {
    async fn add_node(&self, node: Node) -> Result<(), GraphError> {
        let _: Option<Node> = self
            .db
            .create(("node", &node.id))
            .content(node)
            .await
            .map_err(|e| GraphError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn add_edge(&self, edge: Edge) -> Result<(), GraphError> {
        // Validate relation name to prevent injection/errors
        if !edge.relation.chars().all(|c| c.is_alphanumeric() || c == '_') {
             return Err(GraphError::Storage(format!("Invalid relation name: {}", edge.relation)));
        }

        // Use strict SQL for relation creation with properties
        let sql = format!(
            "RELATE node:{}->{}->node:{} SET weight = $weight, partition_id = $partition",
            edge.source, edge.relation, edge.target
        );

        self.db
            .query(sql)
            .bind(("weight", edge.weight))
            .bind(("partition", edge.partition_id))
            .await
            .map_err(|e| GraphError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn get_node(&self, id: &str) -> Result<Node, GraphError> {
        let node: Option<Node> = self
            .db
            .select(("node", id))
            .await
            .map_err(|e| GraphError::Storage(e.to_string()))?;

        node.ok_or(GraphError::NotFound(id.to_string()))
    }

    async fn update_node(&self, node: Node) -> Result<(), GraphError> {
        let _: Option<Node> = self
            .db
            .update(("node", &node.id))
            .content(node)
            .await
            .map_err(|e| GraphError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn get_neighbors(&self, id: &str) -> Result<Vec<(Edge, Node)>, GraphError> {
        // Fetch outgoing edges and their target nodes
        // SELECT ->? as edges FROM node:id FETCH edges.out
        let sql = format!(
            "SELECT ->? as edges FROM node:{} FETCH edges.out",
            id
        );

        let mut response = self
            .db
            .query(sql)
            .await
            .map_err(|e| GraphError::Storage(e.to_string()))?;

        #[derive(Deserialize, Debug)]
        struct RelationEdge {
            #[serde(rename = "in")]
            source: surrealdb::sql::Thing,
            #[serde(rename = "out")]
            target: Node, // Fetched node
            
            // Relation properties
            #[serde(alias = "relation")] // The table name is not directly here in fetched structure usually
            id: surrealdb::sql::Thing,   // The relation ID: table:id
            
            weight: Option<f32>,
            partition_id: Option<String>,
        }

        #[derive(Deserialize, Debug)]
        struct NeighborResult {
            edges: Vec<RelationEdge>,
        }

        // SurrealDB returns a list of results. We expect one result (for the one node we queried)
        let result: Option<NeighborResult> = response
            .take(0)
            .map_err(|e| GraphError::Storage(e.to_string()))?;

        let mut neighbors = Vec::new();

        if let Some(data) = result {
             for rel_edge in data.edges {
                let relation_name = rel_edge.id.tb.clone();
                let edge = Edge {
                    source: rel_edge.source.id.to_string(),
                    target: rel_edge.target.id.to_string(),
                    relation: relation_name,
                    weight: rel_edge.weight.unwrap_or(1.0),
                    partition_id: rel_edge.partition_id.unwrap_or_else(|| "personal".to_string()),
                };
                neighbors.push((edge, rel_edge.target));
             }
        }

        Ok(neighbors)
    }

    async fn query_by_partition(&self, partition_id: &str) -> Result<Vec<Node>, GraphError> {
        let sql = "SELECT * FROM node WHERE partition_id = $partition";
        let pid = partition_id.to_string();

        let mut response = self
            .db
            .query(sql)
            .bind(("partition", pid))
            .await
            .map_err(|e| GraphError::Storage(e.to_string()))?;

        let nodes: Vec<Node> = response
            .take(0)
            .map_err(|e| GraphError::Storage(e.to_string()))?;

        Ok(nodes)
    }

    async fn get_neighbors_in_partition(
        &self,
        id: &str,
        partition_id: &str,
    ) -> Result<Vec<(Edge, Node)>, GraphError> {
        // Optimization: Filter in SQL
        // We select edges where relation property partition_id matches
        // AND the target node partition_id matches.
        // But in SurrealQL for `->?`, filtering on the edge properties is easy, 
        // filtering on target node properties while fetching is also possible.
        
        // sql variable removal
        /*
        let sql = format!(
            "SELECT ->? as edges FROM node:{} WHERE partition_id = $pid FETCH edges.out", 
             id
        );
        */
        // Note: The WHERE clause here applies to the NODE, not the edges, if placed after FROM node:{}.
        // To filter edges... `SELECT ->?(partition_id=$pid) as edges ...` syntax might be intricate.
        // Fallback: Use get_neighbors and filter in Rust for safety and simplicity per ADR-007 (simple over complex query)
        
        let all = self.get_neighbors(id).await?;
        let filtered = all.into_iter()
            .filter(|(e, n)| e.partition_id == partition_id && n.partition_id == partition_id)
            .collect();
            
        Ok(filtered)
    }
}

#[async_trait]
impl VectorStore for SurrealStore {
    async fn add_embedding(&self, id: &str, vector: Vec<f32>) -> Result<(), GraphError> {
        // Update the node with the embedding
        // Assuming 'embedding' field on the node
        let sql = format!("UPDATE node:{} SET embedding = $vector", id);
        self.db
            .query(sql)
            .bind(("vector", vector))
            .await
            .map_err(|e| GraphError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn search(
        &self,
        vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<(String, f32)>, GraphError> {
        // Vector search query
        // SELECT id, vector::similarity::cosine(embedding, $query) as score FROM node ORDER BY score DESC LIMIT $limit
        let sql = "SELECT id, vector::similarity::cosine(embedding, $query) as score FROM node ORDER BY score DESC LIMIT $limit";

        let mut response = self
            .db
            .query(sql)
            .bind(("query", vector))
            .bind(("limit", limit))
            .await
            .map_err(|e| GraphError::Storage(e.to_string()))?;

        // Parse results
        // This requires a struct to deserialize into
        #[derive(Deserialize)]
        struct SearchResult {
            id: surrealdb::sql::Thing,
            score: f32,
        }

        let results: Vec<SearchResult> = response
            .take(0)
            .map_err(|e| GraphError::Storage(e.to_string()))?;

        Ok(results
            .into_iter()
            .map(|r| (r.id.id.to_string(), r.score))
            .collect())
    }
}
