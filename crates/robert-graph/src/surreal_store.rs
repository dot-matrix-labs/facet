use crate::{Edge, GraphError, GraphStore, Node, VectorStore};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Deserialize)]
struct SurrealNode {
    id: surrealdb::sql::Thing,
    label: String,
    properties: serde_json::Value,
    partition_id: String,
}

#[derive(Serialize)]
struct NodeContent {
    label: String,
    properties: serde_json::Value,
    partition_id: String,
}

impl From<SurrealNode> for Node {
    fn from(sn: SurrealNode) -> Self {
        Node {
            id: sn.id.id.to_string(), 
            label: sn.label,
            properties: sn.properties,
            partition_id: sn.partition_id,
        }
    }
}

#[async_trait]
impl GraphStore for SurrealStore {
    async fn add_node(&self, node: Node) -> Result<(), GraphError> {
        let content = NodeContent {
            label: node.label,
            properties: node.properties,
            partition_id: node.partition_id,
        };

        let _: Option<serde::de::IgnoredAny> = self
            .db
            .create(("node", &node.id))
            .content(content)
            .await
            .map_err(|e| GraphError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn add_edge(&self, edge: Edge) -> Result<(), GraphError> {
        // Validate relation name
        if !edge.relation.chars().all(|c| c.is_alphanumeric() || c == '_') {
             return Err(GraphError::Storage(format!("Invalid relation name: {}", edge.relation)));
        }

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
        let node: Option<SurrealNode> = self
            .db
            .select(("node", id))
            .await
            .map_err(|e| GraphError::Storage(e.to_string()))?;

        node.map(Node::from)
            .ok_or(GraphError::NotFound(id.to_string()))
    }

    async fn update_node(&self, node: Node) -> Result<(), GraphError> {
        let content = NodeContent {
            label: node.label,
            properties: node.properties,
            partition_id: node.partition_id,
        };

        let _: Option<serde::de::IgnoredAny> = self
            .db
            .update(("node", &node.id))
            .content(content)
            .await
            .map_err(|e| GraphError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn get_neighbors(&self, id: &str) -> Result<Vec<(Edge, Node)>, GraphError> {
        let sql = format!(
            "SELECT ->? FROM node:{}",
            id
        );

        let mut response = self
            .db
            .query(sql)
            .await
            .map_err(|e| GraphError::Storage(e.to_string()))?;

        // Deserialize into HashMap to capture dynamic keys like "->knows"
        type RelationMap = std::collections::HashMap<String, Vec<surrealdb::sql::Thing>>;

        let result: Option<RelationMap> = response
            .take(0)
            .map_err(|e| GraphError::Storage(format!("Failed to parse relations: {}", e)))?;

        let mut neighbors = Vec::new();

        if let Some(map) = result {
            let mut relation_things = Vec::new();

            // Iterate over keys to extract table names/relations
            for (_key, things) in map {
                 relation_things.extend(things);
            }

            if relation_things.is_empty() {
                return Ok(vec![]);
            }

            // Batch fetch relation records
            #[derive(Deserialize)]
            struct RelationRecord {
                #[serde(alias = "relation")]
                id: surrealdb::sql::Thing,
                #[serde(rename = "in")]
                source: surrealdb::sql::Thing,
                #[serde(rename = "out")]
                target: surrealdb::sql::Thing,
                
                weight: Option<f32>,
                partition_id: Option<String>,
            }

            let rels_sql = "SELECT * FROM $ids";
            let mut rels_response = self
                .db
                .query(rels_sql)
                .bind(("ids", relation_things))
                .await
                .map_err(|e| GraphError::Storage(e.to_string()))?;

            let relations: Vec<RelationRecord> = rels_response
                .take(0)
                .map_err(|e| GraphError::Storage(e.to_string()))?;

            if relations.is_empty() {
                return Ok(vec![]);
            }

            // Collect target IDs for batch fetch
            let target_ids: Vec<surrealdb::sql::Thing> = relations
                .iter()
                .map(|r| r.target.clone())
                .collect();

            // Batch fetch nodes
            let nodes_sql = "SELECT * FROM $ids";
            let mut nodes_response = self
                .db
                .query(nodes_sql)
                .bind(("ids", target_ids))
                .await
                .map_err(|e| GraphError::Storage(e.to_string()))?;

            let nodes: Vec<SurrealNode> = nodes_response
                .take(0)
                .map_err(|e| GraphError::Storage(e.to_string()))?;

            // Map Thing -> Node
            let mut node_map = std::collections::HashMap::new();
            for sn in nodes {
                node_map.insert(sn.id.clone(), Node::from(sn));
            }

            for rel in relations {
                if let Some(target_node) = node_map.get(&rel.target) {
                    let relation_name = rel.id.tb.clone();
                    let edge = Edge {
                        source: id.to_string(), // Use the method argument 'id'
                        target: rel.target.id.to_string(), // Keep ID string
                        relation: relation_name,
                        weight: rel.weight.unwrap_or(1.0),
                        partition_id: rel.partition_id.unwrap_or_else(|| "personal".to_string()),
                    };
                    neighbors.push((edge, target_node.clone()));
                }
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

        let nodes: Vec<SurrealNode> = response
            .take(0)
            .map_err(|e| GraphError::Storage(e.to_string()))?;

        Ok(nodes.into_iter().map(Node::from).collect())
    }

    async fn get_neighbors_in_partition(
        &self,
        id: &str,
        partition_id: &str,
    ) -> Result<Vec<(Edge, Node)>, GraphError> {
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
        let sql = "SELECT id, vector::similarity::cosine(embedding, $query) as score FROM node ORDER BY score DESC LIMIT $limit";

        let mut response = self
            .db
            .query(sql)
            .bind(("query", vector))
            .bind(("limit", limit))
            .await
            .map_err(|e| GraphError::Storage(e.to_string()))?;

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
