//! Database write stress test for operation_logs table
//!
//! A flexible stress test tool to measure maximum write throughput.
//!
//! ## Usage
//!
//! ```bash
//! # Set database connection
//! export DATABASE_URL="postgres://user:password@localhost:5432/ot_test"
//!
//! # Run with defaults (1000 ops, 100 batch size, 4 concurrent writers)
//! cargo run --example stress_test --release
//!
//! # Custom parameters
//! cargo run --example stress_test --release -- \
//!     --total 100000 \
//!     --batch 500 \
//!     --concurrency 8 \
//!     --params-size 1024
//! ```
//!
//! ## Parameters
//!
//! - `--total`: Total number of operations to insert (default: 10000)
//! - `--batch`: Batch size for each COPY operation (default: 100)
//! - `--concurrency`: Number of concurrent writers (default: 4)
//! - `--params-size`: Size of params payload in bytes (default: 512)
//! - `--cleanup`: Clean up data after test (default: true)
//! - `--warmup`: Number of warmup iterations (default: 2)

use chrono::Utc;
use ot_common::copy_writer::{copy_operations, CopyOperationData};
use ot_common::database::entities::documents;
use rand::Rng;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct TestConfig {
    total_operations: usize,
    batch_size: usize,
    concurrency: usize,
    params_size: usize,
    cleanup: bool,
    warmup_iterations: usize,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            total_operations: 10000,
            batch_size: 100,
            concurrency: 4,
            params_size: 512,
            cleanup: true,
            warmup_iterations: 2,
        }
    }
}

impl TestConfig {
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut config = Self::default();

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--total" => {
                    i += 1;
                    config.total_operations = args[i].parse().expect("Invalid --total value");
                }
                "--batch" => {
                    i += 1;
                    config.batch_size = args[i].parse().expect("Invalid --batch value");
                }
                "--concurrency" => {
                    i += 1;
                    config.concurrency = args[i].parse().expect("Invalid --concurrency value");
                }
                "--params-size" => {
                    i += 1;
                    config.params_size = args[i].parse().expect("Invalid --params-size value");
                }
                "--cleanup" => {
                    i += 1;
                    config.cleanup = args[i].parse().expect("Invalid --cleanup value");
                }
                "--warmup" => {
                    i += 1;
                    config.warmup_iterations = args[i].parse().expect("Invalid --warmup value");
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Unknown argument: {}", args[i]);
                    print_help();
                    std::process::exit(1);
                }
            }
            i += 1;
        }

        config
    }
}

fn print_help() {
    println!(
        r#"
Database Write Stress Test for operation_logs

USAGE:
    cargo run --example stress_test --release -- [OPTIONS]

OPTIONS:
    --total <N>         Total operations to insert (default: 10000)
    --batch <N>         Batch size for COPY (default: 100)
    --concurrency <N>   Concurrent writers (default: 4)
    --params-size <N>   Params payload size in bytes (default: 512)
    --cleanup <bool>    Clean up after test (default: true)
    --warmup <N>        Warmup iterations (default: 2)
    -h, --help          Print this help message

ENVIRONMENT:
    DATABASE_URL        PostgreSQL connection string (required)

EXAMPLE:
    export DATABASE_URL="postgres://user:pass@localhost:5432/ot_test"
    cargo run --example stress_test --release -- --total 100000 --batch 500 --concurrency 8
"#
    );
}

/// Create test documents in the database to satisfy foreign key constraints
async fn create_test_documents(db: &DatabaseConnection, count: usize) -> Vec<Uuid> {
    let mut doc_ids = Vec::with_capacity(count);

    for i in 0..count {
        let doc_id = Uuid::new_v4();
        let now = Utc::now();

        let doc = documents::ActiveModel {
            id: Set(doc_id),
            name: Set(format!("stress_test_doc_{}", i)),
            creator_id: Set("stress_test".to_string()),
            doc_type: Set(1), // Spreadsheet
            create_type: Set(1),
            current_version: Set(0),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };

        doc.insert(db).await.expect("Failed to create test document");
        doc_ids.push(doc_id);
    }

    doc_ids
}

/// Generate random operation data for testing
fn generate_batch(
    batch_size: usize,
    params_size: usize,
    batch_id: usize,
    doc_ids: &[Uuid],
) -> Vec<CopyOperationData> {
    let mut rng = rand::thread_rng();
    // Use one of the pre-created doc_ids
    let doc_id = doc_ids[batch_id % doc_ids.len()];

    (0..batch_size)
        .map(|i| {
            let params: Vec<u8> = (0..params_size).map(|_| rng.gen()).collect();

            CopyOperationData {
                doc_id,
                rev: (batch_id * batch_size + i) as i64 + 1,
                user_id: format!("user_{}", rng.gen::<u32>() % 1000),
                mutation_id: Uuid::new_v4().to_string(),
                storage_id: None,
                params: Some(params),
                op_id: Uuid::new_v4().to_string(),
                created_at: Utc::now(),
            }
        })
        .collect()
}

async fn cleanup_tables(db: &DatabaseConnection) {
    use sea_orm::ConnectionTrait;
    // Delete operation_logs first (child), then documents (parent) due to FK
    let _ = db
        .execute_unprepared("TRUNCATE TABLE operation_logs RESTART IDENTITY CASCADE")
        .await;
    // Clean up stress test documents
    let _ = db
        .execute_unprepared("DELETE FROM documents WHERE name LIKE 'stress_test_doc_%'")
        .await;
}

#[derive(Debug)]
struct TestResult {
    total_operations: u64,
    total_time: Duration,
    ops_per_second: f64,
    mb_per_second: f64,
    avg_batch_time_ms: f64,
    p50_batch_time_ms: f64,
    p95_batch_time_ms: f64,
    p99_batch_time_ms: f64,
}

async fn run_stress_test(
    db: Arc<DatabaseConnection>,
    config: &TestConfig,
    doc_ids: Arc<Vec<Uuid>>,
) -> TestResult {
    let num_batches = config.total_operations / config.batch_size;
    let semaphore = Arc::new(Semaphore::new(config.concurrency));
    let completed = Arc::new(AtomicU64::new(0));
    let batch_times = Arc::new(tokio::sync::Mutex::new(Vec::with_capacity(num_batches)));

    println!("\n=== Starting Stress Test ===");
    println!("Total operations: {}", config.total_operations);
    println!("Batch size: {}", config.batch_size);
    println!("Number of batches: {}", num_batches);
    println!("Concurrency: {}", config.concurrency);
    println!("Params size: {} bytes", config.params_size);
    println!("Test documents: {}", doc_ids.len());
    println!();

    let start = Instant::now();
    let progress_start = start;

    // Progress reporter task
    let completed_clone = Arc::clone(&completed);
    let total = num_batches as u64;
    let progress_handle = tokio::spawn(async move {
        let mut last_count = 0u64;
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let current = completed_clone.load(Ordering::Relaxed);
            if current >= total {
                break;
            }
            let elapsed = progress_start.elapsed().as_secs_f64();
            let rate = (current - last_count) as f64;
            let overall_rate = current as f64 / elapsed;
            println!(
                "Progress: {}/{} batches ({:.1}%) | Current: {:.0} batches/s | Avg: {:.0} batches/s",
                current,
                total,
                current as f64 / total as f64 * 100.0,
                rate,
                overall_rate
            );
            last_count = current;
        }
    });

    // Spawn batch writers
    let mut handles = Vec::with_capacity(num_batches);

    for batch_id in 0..num_batches {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let db = Arc::clone(&db);
        let completed = Arc::clone(&completed);
        let batch_times = Arc::clone(&batch_times);
        let doc_ids = Arc::clone(&doc_ids);
        let params_size = config.params_size;
        let batch_size = config.batch_size;

        handles.push(tokio::spawn(async move {
            let batch = generate_batch(batch_size, params_size, batch_id, &doc_ids);
            let batch_start = Instant::now();

            match copy_operations(&db, &batch).await {
                Ok(_) => {
                    let elapsed = batch_start.elapsed();
                    batch_times.lock().await.push(elapsed);
                    completed.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    eprintln!("Batch {} failed: {:?}", batch_id, e);
                }
            }

            drop(permit);
        }));
    }

    // Wait for all batches
    for handle in handles {
        let _ = handle.await;
    }

    let total_time = start.elapsed();
    progress_handle.abort();

    // Calculate statistics
    let mut times: Vec<Duration> = batch_times.lock().await.clone();
    times.sort();

    let total_ops = completed.load(Ordering::Relaxed) * config.batch_size as u64;
    let total_bytes = total_ops * config.params_size as u64;
    let ops_per_second = total_ops as f64 / total_time.as_secs_f64();
    let mb_per_second = total_bytes as f64 / 1024.0 / 1024.0 / total_time.as_secs_f64();

    let avg_batch_time = times.iter().sum::<Duration>() / times.len() as u32;
    let p50 = times[times.len() / 2];
    let p95 = times[(times.len() as f64 * 0.95) as usize];
    let p99 = times[(times.len() as f64 * 0.99) as usize];

    TestResult {
        total_operations: total_ops,
        total_time,
        ops_per_second,
        mb_per_second,
        avg_batch_time_ms: avg_batch_time.as_secs_f64() * 1000.0,
        p50_batch_time_ms: p50.as_secs_f64() * 1000.0,
        p95_batch_time_ms: p95.as_secs_f64() * 1000.0,
        p99_batch_time_ms: p99.as_secs_f64() * 1000.0,
    }
}

fn print_results(result: &TestResult) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    STRESS TEST RESULTS                       ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Total Operations:     {:>15}                      ║",
        result.total_operations
    );
    println!(
        "║  Total Time:           {:>15.2} s                    ║",
        result.total_time.as_secs_f64()
    );
    println!(
        "║                                                              ║"
    );
    println!(
        "║  Throughput:           {:>15.0} ops/s                 ║",
        result.ops_per_second
    );
    println!(
        "║  Data Rate:            {:>15.2} MB/s                  ║",
        result.mb_per_second
    );
    println!(
        "║                                                              ║"
    );
    println!("║  Batch Latency:                                              ║");
    println!(
        "║    Average:            {:>15.2} ms                    ║",
        result.avg_batch_time_ms
    );
    println!(
        "║    P50:                {:>15.2} ms                    ║",
        result.p50_batch_time_ms
    );
    println!(
        "║    P95:                {:>15.2} ms                    ║",
        result.p95_batch_time_ms
    );
    println!(
        "║    P99:                {:>15.2} ms                    ║",
        result.p99_batch_time_ms
    );
    println!("╚══════════════════════════════════════════════════════════════╝");
}

#[tokio::main]
async fn main() {
    // Load .env if present
    dotenvy::dotenv().ok();

    let config = TestConfig::from_args();

    // Connect to database
    let database_url = std::env::var("DATABASE_URL").expect(
        "DATABASE_URL environment variable must be set.\n\
         Example: export DATABASE_URL=\"postgres://user:pass@localhost:5432/ot_test\"",
    );

    println!("Connecting to database...");
    let db = Arc::new(
        ot_common::database::connect(&database_url)
            .await
            .expect("Failed to connect to database"),
    );
    println!("Connected!");

    // Initial cleanup
    println!("Cleaning up existing data...");
    cleanup_tables(&db).await;

    // Create test documents to satisfy foreign key constraints
    // Use enough documents to distribute load (1 doc per concurrent writer minimum)
    let num_docs = config.concurrency.max(10);
    println!("Creating {} test documents...", num_docs);
    let doc_ids = Arc::new(create_test_documents(&db, num_docs).await);
    println!("Test documents created!");

    // Warmup
    if config.warmup_iterations > 0 {
        println!("\n=== Warmup ({} iterations) ===", config.warmup_iterations);
        for i in 0..config.warmup_iterations {
            let batch = generate_batch(config.batch_size, config.params_size, i, &doc_ids);
            let start = Instant::now();
            copy_operations(&db, &batch).await.expect("Warmup failed");
            println!(
                "Warmup {}: {} ops in {:.2}ms",
                i + 1,
                config.batch_size,
                start.elapsed().as_secs_f64() * 1000.0
            );
        }
        // Only truncate operation_logs, keep documents for main test
        {
            use sea_orm::ConnectionTrait;
            let _ = db
                .execute_unprepared("TRUNCATE TABLE operation_logs RESTART IDENTITY")
                .await;
        }
    }

    // Run stress test
    let result = run_stress_test(Arc::clone(&db), &config, Arc::clone(&doc_ids)).await;
    print_results(&result);

    // Cleanup
    if config.cleanup {
        println!("\nCleaning up test data...");
        cleanup_tables(&db).await;
        println!("Done!");
    } else {
        println!("\nTest data retained in database.");
    }
}
