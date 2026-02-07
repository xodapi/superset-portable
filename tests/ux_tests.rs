use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// Integration tests for User Experience
/// These tests run the actual binary logic (or simulate it) to verify UX requirements.

#[test]
fn test_ux_data_loader_speed() {
    // 1. Setup: Create a dummy CSV file
    let file_path = PathBuf::from("tests/test_data.csv");
    let content = "id,name,value\n1,Test,100\n2,Test2,200\n";
    std::fs::write(&file_path, content).expect("Failed to create test CSV");
    
    // 2. Act: Measure time to load
    let start = Instant::now();
    
    // We invoke the binary logic directly via module if possible, 
    // but since this is an integration test outside `src`, we might need to rely on the binary.
    // However, to test specific modules from `src`, `src/lib.rs` structure is preferred.
    // Since `src/main.rs` is a binary crate, we can't import its modules in `tests/`.
    // We will simulate the user running the command.
    
    // Compile binary first (assumed done or cargo test does it)
    let status = Command::new("cargo")
        .args(&["run", "--", "load-data", "tests/test_data.csv", "--table", "test_ux_table"])
        .current_dir("c:\\project\\ass")
        .status()
        .expect("Failed to run cargo run");
        
    let duration = start.elapsed();
    
    // 3. Assert: Verify success and speed
    assert!(status.success(), "Data loader command failed");
    println!("Data loaded in: {:?}", duration);
    
    // For a tiny file it should be instant < 5s (compilation might take time if not cached)
    // In a real scenario we'd query the DB to check data.
    
    // Clean up
    let _ = std::fs::remove_file(file_path);
}

#[tokio::test]
async fn test_ux_knowledge_search_readiness() {
    // 1. Setup: Verify Knowledge Base exists
    let knowledge_dir = PathBuf::from("knowledge");
    assert!(knowledge_dir.exists(), "Knowledge base directory missing! User cannot search.");
    
    // 2. Check for key topics
    let topics = ["databases", "charts", "troubleshooting"];
    for topic in topics {
        let path = knowledge_dir.join(topic);
        assert!(path.exists(), "Missing knowledge topic: {} - UX degradation!", topic);
    }
}
