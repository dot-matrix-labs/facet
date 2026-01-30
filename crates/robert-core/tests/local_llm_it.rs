use robert_core::llm::LocalLlm;

#[tokio::test]
#[ignore] // Ignored by default as it downloads model (~2GB) and requires Metal/CPU
async fn test_local_llm_generation() {
    let mut llm = LocalLlm::new().expect("Failed to initialize Local LLM");
    
    // Test Summary
    let text = "Robert is a privacy-focused AI assistant that runs locally on your device. It uses GraphRAG to provide context-aware answers without sending your data to the cloud.";
    let summary = llm.synthesize(text).expect("Failed to synthesize");
    println!("Summary: {}", summary);
    assert!(!summary.is_empty());

    // Test PII Extraction
    let pii_text = "Contact Alice at alice@example.com or call 555-0123.";
    let (redacted, pii_map) = llm.extract_pii(pii_text).expect("Failed to extract PII");
    println!("Redacted: {}", redacted);
    println!("PII Map: {:?}", pii_map);
    
    assert!(redacted.contains("[")); // Should contain placeholders
    assert!(!pii_map.is_empty());
}
