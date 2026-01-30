use robert_graph::{GraphStore, Node, Edge, VectorStore};
use robert_graph::surreal_store::SurrealStore;
use serde_json::json;
use tempfile::tempdir;

#[tokio::test]
async fn test_surreal_graph_ops() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    
    let store = SurrealStore::new(db_path).await.unwrap();

    // 1. Add Node
    let node1 = Node {
        id: "p1".to_string(),
        label: "Person".to_string(),
        properties: json!({"name": "Alice"}),
        partition_id: "personal".to_string(),
    };
    store.add_node(node1.clone()).await.unwrap();

    let node2 = Node {
        id: "p2".to_string(),
        label: "Person".to_string(),
        properties: json!({"name": "Bob"}),
        partition_id: "work".to_string(),
    };
    store.add_node(node2.clone()).await.unwrap();

    // 2. Get Node
    let retrieved = store.get_node("p1").await.unwrap();
    assert_eq!(retrieved.id, "p1");
    assert_eq!(retrieved.properties["name"], "Alice");

    // 3. Add Edge
    let edge = Edge {
        source: "p1".to_string(),
        target: "p2".to_string(),
        relation: "knows".to_string(),
        weight: 0.8,
        partition_id: "personal".to_string(),
    };
    store.add_edge(edge.clone()).await.unwrap();

    // 4. Get Neighbors
    let neighbors = store.get_neighbors("p1").await.unwrap();
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].1.id, "p2");
    assert_eq!(neighbors[0].0.relation, "knows");

    // 5. Query by Partition
    let personal_nodes = store.query_by_partition("personal").await.unwrap();
    assert_eq!(personal_nodes.len(), 1);
    assert_eq!(personal_nodes[0].id, "p1");

    let work_nodes = store.query_by_partition("work").await.unwrap();
    assert_eq!(work_nodes.len(), 1);
    assert_eq!(work_nodes[0].id, "p2");
}

#[tokio::test]
async fn test_surreal_vector_ops() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_vec.db");
    
    let store = SurrealStore::new(db_path).await.unwrap();

    // Create a node first (embeddings usually attach to nodes)
    let node = Node {
        id: "doc1".to_string(),
        label: "Document".to_string(),
        properties: json!({"content": "Hello world"}),
        partition_id: "personal".to_string(),
    };
    store.add_node(node).await.unwrap();

    // Add embedding
    let vec = vec![1.0, 0.0, 0.5];
    store.add_embedding("doc1", vec.clone()).await.unwrap();

    // Search
    let results = store.search(vec, 1).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "doc1");
    // Cosine similarity of identical vectors should be ~1.0
    assert!((results[0].1 - 1.0).abs() < 0.001);
}
