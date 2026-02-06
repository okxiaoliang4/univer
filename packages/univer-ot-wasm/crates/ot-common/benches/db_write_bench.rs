//! Database write performance benchmark for operation_logs table
//!
//! This benchmark tests different write strategies:
//! 1. Single INSERT (SeaORM) - one record at a time
//! 2. Batch INSERT (SeaORM) - multiple records in one query
//! 3. COPY (PostgreSQL) - bulk insert using COPY protocol
//!
//! ## Running the benchmark
//!
//! Make sure you have a PostgreSQL database running and set the DATABASE_URL:
//!
//! ```bash
//! export DATABASE_URL="postgres://user:password@localhost:5432/ot_test"
//! cargo bench --bench db_write_bench
//! ```
//!
//! Or create a `.env` file in the crate root with:
//! ```
//! DATABASE_URL=postgres://user:password@localhost:5432/ot_test
//! ```

use chrono::Utc;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ot_common::copy_writer::{copy_operations, CopyOperationData};
use ot_common::database::entities::{documents, operation_log};
use rand::Rng;
use sea_orm::{ActiveModelTrait, ConnectionTrait, DatabaseConnection, EntityTrait, Set};
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;

/// Create test documents in the database to satisfy foreign key constraints
async fn create_test_documents(db: &DatabaseConnection, count: usize) -> Vec<Uuid> {
    let mut doc_ids = Vec::with_capacity(count);
    let now = Utc::now();

    for i in 0..count {
        let doc_id = Uuid::new_v4();

        let doc = documents::ActiveModel {
            id: Set(doc_id),
            name: Set(format!("bench_test_doc_{}", i)),
            creator_id: Set("benchmark".to_string()),
            doc_type: Set(1),
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

/// Generate random operation data for testing using pre-created doc_ids
fn generate_operation_data(
    count: usize,
    params_size: usize,
    doc_ids: &[Uuid],
) -> Vec<CopyOperationData> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|i| {
            let params: Vec<u8> = (0..params_size).map(|_| rng.gen()).collect();
            let doc_id = doc_ids[i % doc_ids.len()];

            CopyOperationData {
                doc_id,
                rev: i as i64 + 1,
                user_id: format!("user_{}", rng.gen::<u32>()),
                mutation_id: Uuid::new_v4().to_string(),
                storage_id: None,
                params: Some(params),
                op_id: Uuid::new_v4().to_string(),
                created_at: Utc::now(),
            }
        })
        .collect()
}

/// Convert CopyOperationData to SeaORM ActiveModel
fn to_active_model(data: &CopyOperationData) -> operation_log::ActiveModel {
    operation_log::ActiveModel {
        id: Default::default(),
        doc_id: Set(data.doc_id),
        rev: Set(data.rev),
        user_id: Set(data.user_id.clone()),
        mutation_id: Set(data.mutation_id.clone()),
        storage_id: Set(data.storage_id),
        params: Set(data.params.clone()),
        op_id: Set(data.op_id.clone()),
        created_at: Set(data.created_at.into()),
    }
}

/// Clean up test data after benchmark
async fn cleanup_test_data(db: &DatabaseConnection) {
    let _ = db
        .execute_unprepared("TRUNCATE TABLE operation_logs RESTART IDENTITY CASCADE")
        .await;
    let _ = db
        .execute_unprepared("DELETE FROM documents WHERE name LIKE 'bench_test_doc_%'")
        .await;
}

/// Benchmark single INSERT operations
async fn bench_single_insert(db: &DatabaseConnection, data: &[CopyOperationData]) {
    for op in data {
        let model = to_active_model(op);
        model.insert(db).await.expect("Failed to insert");
    }
}

/// Benchmark batch INSERT operations using SeaORM
async fn bench_batch_insert(db: &DatabaseConnection, data: &[CopyOperationData]) {
    let models: Vec<operation_log::ActiveModel> = data.iter().map(to_active_model).collect();
    operation_log::Entity::insert_many(models)
        .exec(db)
        .await
        .expect("Failed to batch insert");
}

/// Benchmark COPY operations
async fn bench_copy_insert(db: &DatabaseConnection, data: &[CopyOperationData]) {
    copy_operations(db, data)
        .await
        .expect("Failed to COPY insert");
}

/// Create database connection from environment
async fn create_db_connection() -> DatabaseConnection {
    dotenvy::dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for benchmarks");
    ot_common::database::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}

fn bench_write_methods(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let db = rt.block_on(async { Arc::new(create_db_connection().await) });

    // Create test documents
    let doc_ids = rt.block_on(async { create_test_documents(&db, 10).await });

    let batch_sizes = [10, 50, 100, 500, 1000];
    let params_size = 512;

    let mut group = c.benchmark_group("db_write");
    group.sample_size(10);

    for size in batch_sizes {
        let data = generate_operation_data(size, params_size, &doc_ids);

        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::new("copy", size), &data, |b, data| {
            let db = Arc::clone(&db);
            b.to_async(&rt).iter(|| async {
                bench_copy_insert(&db, data).await;
            });
        });

        if size <= 500 {
            group.bench_with_input(BenchmarkId::new("batch_insert", size), &data, |b, data| {
                let db = Arc::clone(&db);
                b.to_async(&rt).iter(|| async {
                    bench_batch_insert(&db, data).await;
                });
            });
        }

        if size <= 50 {
            group.bench_with_input(BenchmarkId::new("single_insert", size), &data, |b, data| {
                let db = Arc::clone(&db);
                b.to_async(&rt).iter(|| async {
                    bench_single_insert(&db, data).await;
                });
            });
        }
    }

    group.finish();

    rt.block_on(async {
        cleanup_test_data(&db).await;
    });
}

fn bench_copy_with_different_params_sizes(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let db = rt.block_on(async { Arc::new(create_db_connection().await) });
    let doc_ids = rt.block_on(async { create_test_documents(&db, 10).await });

    let mut group = c.benchmark_group("copy_params_size");
    group.sample_size(10);

    let batch_size = 100;
    let params_sizes = [128, 512, 1024, 2048, 4096];

    for params_size in params_sizes {
        let data = generate_operation_data(batch_size, params_size, &doc_ids);
        let total_bytes = batch_size * params_size;

        group.throughput(Throughput::Bytes(total_bytes as u64));

        group.bench_with_input(
            BenchmarkId::new("copy", format!("{}B", params_size)),
            &data,
            |b, data| {
                let db = Arc::clone(&db);
                b.to_async(&rt).iter(|| async {
                    bench_copy_insert(&db, data).await;
                });
            },
        );
    }

    group.finish();

    rt.block_on(async {
        cleanup_test_data(&db).await;
    });
}

fn bench_concurrent_writes(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let db = rt.block_on(async { Arc::new(create_db_connection().await) });
    let doc_ids = rt.block_on(async { create_test_documents(&db, 20).await });

    let mut group = c.benchmark_group("concurrent_writes");
    group.sample_size(10);

    let batch_size = 100;
    let params_size = 512;
    let concurrency_levels = [1, 2, 4, 8, 16];

    for concurrency in concurrency_levels {
        let all_data: Vec<Vec<CopyOperationData>> = (0..concurrency)
            .map(|_| generate_operation_data(batch_size, params_size, &doc_ids))
            .collect();

        group.throughput(Throughput::Elements((batch_size * concurrency) as u64));

        group.bench_with_input(
            BenchmarkId::new("copy", format!("{}x{}", concurrency, batch_size)),
            &all_data,
            |b, all_data| {
                let db = Arc::clone(&db);
                b.to_async(&rt).iter(|| {
                    let db = Arc::clone(&db);
                    let data = all_data.clone();
                    async move {
                        let handles: Vec<_> = data
                            .into_iter()
                            .map(|batch| {
                                let db = Arc::clone(&db);
                                tokio::spawn(async move {
                                    copy_operations(&db, &batch).await.expect("COPY failed");
                                })
                            })
                            .collect();

                        for handle in handles {
                            handle.await.expect("Task panicked");
                        }
                    }
                });
            },
        );
    }

    group.finish();

    rt.block_on(async {
        cleanup_test_data(&db).await;
    });
}

fn bench_sustained_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let db = rt.block_on(async { Arc::new(create_db_connection().await) });
    let doc_ids = rt.block_on(async { create_test_documents(&db, 20).await });

    let mut group = c.benchmark_group("sustained_throughput");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(30));

    let batch_size = 1000;
    let params_size = 512;
    let iterations = 10;

    let all_data: Vec<Vec<CopyOperationData>> = (0..iterations)
        .map(|_| generate_operation_data(batch_size, params_size, &doc_ids))
        .collect();

    group.throughput(Throughput::Elements((batch_size * iterations) as u64));

    group.bench_with_input(
        BenchmarkId::new("copy", format!("{}x{}", iterations, batch_size)),
        &all_data,
        |b, all_data| {
            let db = Arc::clone(&db);
            b.to_async(&rt).iter(|| async {
                for batch in all_data {
                    copy_operations(&db, batch).await.expect("COPY failed");
                }
            });
        },
    );

    group.finish();

    rt.block_on(async {
        cleanup_test_data(&db).await;
    });
}

criterion_group!(
    benches,
    bench_write_methods,
    bench_copy_with_different_params_sizes,
    bench_concurrent_writes,
    bench_sustained_throughput,
);

criterion_main!(benches);
