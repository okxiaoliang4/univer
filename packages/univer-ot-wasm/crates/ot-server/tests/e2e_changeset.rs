mod common;

use anyhow::{Context, Result};
use common::{
    create_changeset_request, generate_multiple_set_range_values, print_mutation_stats,
    MemoryMonitor, PerfTimer, SocketIOTestClient,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// Configuration for the e2e test
#[derive(Debug, Clone)]
struct TestConfig {
    server_url: String,
    socketio_url: String,
    cell_count_per_mutation: usize,
    mutation_count: usize,
    user_id: String,
    client_id: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            server_url: std::env::var("OT_SERVER_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            socketio_url: std::env::var("OT_SOCKETIO_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            cell_count_per_mutation: std::env::var("CELL_COUNT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(20000),
            mutation_count: std::env::var("MUTATION_COUNT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3),
            user_id: "test_user_001".to_string(),
            client_id: "test_client_001".to_string(),
        }
    }
}

/// Request structure for creating a document via REST API
#[derive(Debug, Serialize)]
struct CreateDocumentRequest {
    doc_id: String,
    creator_id: String,
    name: String,
    doc_type: i16,
    create_type: i16,
    content: serde_json::Value,
}

/// Response structure for document creation
#[derive(Debug, Deserialize)]
struct CreateDocumentResponse {
    doc_id: String,
    version: i64,
}

/// Create a new document via REST API
async fn create_document(
    server_url: &str,
    doc_id: &str,
    creator_id: &str,
) -> Result<CreateDocumentResponse> {
    let client = reqwest::Client::new();

    // Create initial empty workbook content
    let content = json!({
        "id": doc_id,
        "name": "E2E Test Workbook",
        "appVersion": "0.5.0",
        "locale": "en-US",
        "sheets": {
            "sheet1": {
                "id": "sheet1",
                "name": "Sheet1",
                "rowCount": 1000,
                "columnCount": 20,
                "cellData": {},
                "rowData": {},
                "columnData": {},
            }
        },
        "resources": []
    });

    let request = CreateDocumentRequest {
        doc_id: doc_id.to_string(),
        creator_id: creator_id.to_string(),
        name: "E2E Test Document".to_string(),
        doc_type: 2, // SHEET
        create_type: 1,
        content,
    };

    println!("📝 Creating document via REST API: {}", doc_id);

    let response = client
        .post(format!("{}/api/documents", server_url))
        .json(&request)
        .send()
        .await
        .context("Failed to send create document request")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "Failed to create document: {} - {}",
            status,
            body
        );
    }

    let result: CreateDocumentResponse = response
        .json()
        .await
        .context("Failed to parse create document response")?;

    println!("✅ Document created: {} (version: {})", result.doc_id, result.version);

    Ok(result)
}

/// Main e2e test for changeset with memory monitoring
#[tokio::test]
#[ignore] // Remove this to run the test
async fn test_e2e_changeset_with_memory_monitoring() -> Result<()> {
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║  E2E Changeset Test with Memory Monitoring           ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");

    let config = TestConfig::default();

    println!("📋 Test Configuration:");
    println!("   Server URL: {}", config.server_url);
    println!("   Socket.IO URL: {}", config.socketio_url);
    println!("   Cells per mutation: {}", config.cell_count_per_mutation);
    println!("   Mutation count: {}", config.mutation_count);
    println!("   User ID: {}", config.user_id);
    println!("   Client ID: {}", config.client_id);
    println!();

    // Initialize memory monitor
    let mut memory_monitor = MemoryMonitor::new("test_start");
    let logging_handle = memory_monitor.start_periodic_logging();

    // Step 1: Create document
    let doc_id = Uuid::new_v4().to_string();
    {
        let _timer = PerfTimer::start("Create Document");
        create_document(&config.server_url, &doc_id, &config.user_id)
            .await
            .context("Failed to create document")?;
    }
    memory_monitor.record("after_document_creation");

    // Wait a bit for document to be ready
    sleep(Duration::from_millis(500)).await;

    // Step 2: Connect to Socket.IO server
    let client = {
        let _timer = PerfTimer::start("Socket.IO Connection");
        SocketIOTestClient::connect(&config.socketio_url)
            .await
            .context("Failed to connect to Socket.IO server")?
    };
    memory_monitor.record("after_socketio_connection");

    // Step 3: Join document room
    {
        let _timer = PerfTimer::start("Join Document");
        let ack = client
            .join_doc(&doc_id)
            .await
            .context("Failed to join document")?;

        if ack.status != "ok" {
            anyhow::bail!("Failed to join document: {:?}", ack.message);
        }

        println!("📄 Joined document, version: {:?}", ack.version);
    }
    memory_monitor.record("after_join_doc");

    // Step 4: Generate mutations
    let mutations = {
        let _timer = PerfTimer::start("Generate Mutations");
        generate_multiple_set_range_values(
            &doc_id,
            "sheet1",
            config.mutation_count,
            config.cell_count_per_mutation,
            &config.client_id,
        )
    };
    memory_monitor.record("after_mutation_generation");

    print_mutation_stats(&mutations);

    // Step 5: Send changesets
    println!("\n═══════════════════════════════════════════════════════");
    println!("🚀 Starting Changeset Transmission");
    println!("═══════════════════════════════════════════════════════\n");

    let mut current_rev = 1i64; // Start from version 1 (initial snapshot)

    for (i, mutation) in mutations.into_iter().enumerate() {
        let batch_label = format!("changeset_{}", i + 1);
        let timer = PerfTimer::start(&batch_label);

        let request = create_changeset_request(
            &doc_id,
            current_rev,
            &config.client_id,
            vec![mutation],
        );

        let ack = client
            .send_changeset(request)
            .await
            .context(format!("Failed to send changeset {}", i + 1))?;

        if ack.status != "ok" {
            anyhow::bail!(
                "Changeset {} failed: {:?}",
                i + 1,
                ack.message
            );
        }

        if let Some(server_rev) = ack.server_rev {
            current_rev = server_rev;
        }

        timer.stop_with_message(&format!("serverRev={}", current_rev));
        memory_monitor.record(&format!("after_{}", batch_label));

        // Small delay between changesets to avoid overwhelming the server
        if i < config.mutation_count - 1 {
            sleep(Duration::from_millis(100)).await;
        }
    }

    println!("\n═══════════════════════════════════════════════════════");
    println!("✅ All Changesets Sent Successfully");
    println!("═══════════════════════════════════════════════════════\n");

    // Step 6: Wait a bit for server to process
    sleep(Duration::from_secs(2)).await;
    memory_monitor.record("after_processing_complete");

    // Step 7: Disconnect
    {
        let _timer = PerfTimer::start("Disconnect");
        client
            .disconnect()
            .await
            .context("Failed to disconnect")?;
    }
    memory_monitor.record("after_disconnect");

    // Stop periodic logging
    memory_monitor.stop_periodic_logging();
    let _ = logging_handle.await;

    // Print final summary
    memory_monitor.print_summary();

    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║  Test Completed Successfully                          ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");

    Ok(())
}

/// Stress test with multiple concurrent clients
#[tokio::test]
#[ignore] // Remove this to run the test
async fn test_e2e_concurrent_clients() -> Result<()> {
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║  E2E Concurrent Clients Stress Test                   ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");

    let config = TestConfig::default();
    let client_count = 3;

    // Initialize memory monitor
    let mut memory_monitor = MemoryMonitor::new("concurrent_test_start");
    let logging_handle = memory_monitor.start_periodic_logging();

    // Create document
    let doc_id = Uuid::new_v4().to_string();
    create_document(&config.server_url, &doc_id, &config.user_id).await?;
    memory_monitor.record("after_document_creation");

    sleep(Duration::from_millis(500)).await;

    // Spawn multiple clients
    let mut handles = vec![];

    for client_id in 0..client_count {
        let doc_id_clone = doc_id.clone();
        let config_clone = config.clone();

        let handle = tokio::spawn(async move {
            let client_name = format!("client_{}", client_id);
            println!("🔌 {} connecting...", client_name);

            let client = SocketIOTestClient::connect(&config_clone.socketio_url)
                .await?;

            client.join_doc(&doc_id_clone).await?;
            println!("✅ {} joined document", client_name);

            // Each client sends smaller mutations
            let mutations = generate_multiple_set_range_values(
                &doc_id_clone,
                "sheet1",
                2, // 2 mutations per client
                1000, // 1000 cells each
                &client_name,
            );

            for mutation in mutations {
                let request = create_changeset_request(
                    &doc_id_clone,
                    1,
                    &client_name,
                    vec![mutation],
                );

                let ack = client.send_changeset(request).await?;
                if ack.status != "ok" {
                    anyhow::bail!("{} changeset failed", client_name);
                }

                sleep(Duration::from_millis(50)).await;
            }

            client.disconnect().await?;
            println!("🎉 {} completed", client_name);

            Ok::<_, anyhow::Error>(())
        });

        handles.push(handle);
    }

    // Wait for all clients
    for handle in handles {
        handle.await??;
    }

    memory_monitor.record("after_all_clients_complete");

    // Stop periodic logging
    memory_monitor.stop_periodic_logging();
    let _ = logging_handle.await;

    memory_monitor.print_summary();

    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║  Concurrent Test Completed Successfully               ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");

    Ok(())
}
