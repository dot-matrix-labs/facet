use robert_downloader::model_downloader::download_model;
use tempfile::TempDir;

#[tokio::test]
async fn test_download_with_public_model() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("test_model");

    // Test downloading a small public model
    // We use a very small model to avoid long download times
    let result = download_model(
        "hf-internal-testing/tiny-random-gpt2",
        output_path.to_str().unwrap(),
        None,
        false,
        true, // use_xet = true
    )
    .await;

    // The download should succeed
    match result {
        Ok(_) => {
            // Check that some files were downloaded
            let model_dir = output_path.join("hf-internal-testing_tiny-random-gpt2");
            let entries: Vec<_> = std::fs::read_dir(&model_dir)
                .expect("Failed to read model dir")
                .collect();
            assert!(
                !entries.is_empty(),
                "No files were downloaded to output directory"
            );

            // Check for specific expected files
            let file_names: Vec<String> = entries
                .iter()
                .map(|e| {
                    e.as_ref()
                        .unwrap()
                        .file_name()
                        .to_string_lossy()
                        .to_string()
                })
                .collect();

            println!("Downloaded files: {:?}", file_names);
            assert!(file_names.iter().any(|f| f.contains("config.json")));
        }
        Err(e) => {
            panic!("Download failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_download_with_invalid_model() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("nonexistent_model");

    // Test with a model that definitely doesn't exist
    let result = download_model(
        "definitely/does-not-exist-model-12345",
        output_path.to_str().unwrap(),
        None,
        false,
        true,
    )
    .await;

    // This should fail
    assert!(
        result.is_err(),
        "Expected download of nonexistent model to fail"
    );
}
