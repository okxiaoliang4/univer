# Quick Start: E2E Changeset Tests

## Prerequisites

1. **Start PostgreSQL**:
   ```bash
   # Example using Docker
   docker run -d --name postgres \
     -e POSTGRES_DB=univer_ot \
     -e POSTGRES_USER=postgres \
     -e POSTGRES_PASSWORD=postgres \
     -p 5432:5432 \
     postgres:15
   ```

2. **Start Redis**:
   ```bash
   docker run -d --name redis -p 6379:6379 redis:7
   ```

3. **Start MinIO (S3-compatible storage)**:
   ```bash
   docker run -d --name minio \
     -p 9000:9000 -p 9001:9001 \
     -e MINIO_ROOT_USER=minioadmin \
     -e MINIO_ROOT_PASSWORD=minioadmin \
     minio/minio server /data --console-address ":9001"
   ```

4. **Run Migrations**:
   ```bash
   cd ../../migration
   cargo run
   cd ../crates/ot-server
   ```

5. **Configure Environment** (create `.env` file):
   ```bash
   DATABASE_URL=postgresql://postgres:postgres@localhost/univer_ot
   REDIS_URL=redis://localhost:6379
   S3_ENDPOINT=http://localhost:9000
   S3_REGION=us-east-1
   S3_BUCKET=univer-snapshots
   S3_ACCESS_KEY_ID=minioadmin
   S3_SECRET_ACCESS_KEY=minioadmin
   SERVER_PORT=3000
   SNAPSHOT_INTERVAL=50
   ```

6. **Create S3 Bucket**:
   ```bash
   # Install mc (MinIO client)
   brew install minio/stable/mc  # macOS
   # or: sudo apt install minio-client  # Linux

   # Configure mc
   mc alias set local http://localhost:9000 minioadmin minioadmin

   # Create bucket
   mc mb local/univer-snapshots
   ```

## Running the Tests

### Option 1: Using the Test Script (Recommended)

1. **Start the OT Server** (in one terminal):
   ```bash
   cargo run --release
   ```

2. **Run the E2E Tests** (in another terminal):
   ```bash
   # Default test (20,000 cells, 3 mutations)
   ./run_e2e_test.sh

   # Custom configuration
   CELL_COUNT=10000 MUTATION_COUNT=2 ./run_e2e_test.sh

   # Run concurrent clients test
   TEST_NAME=test_e2e_concurrent_clients ./run_e2e_test.sh
   ```

### Option 2: Using Cargo Directly

1. **Start the OT Server** (in one terminal):
   ```bash
   cargo run --release
   ```

2. **Run Tests** (in another terminal):
   ```bash
   # Main e2e test
   cargo test --test e2e_changeset test_e2e_changeset_with_memory_monitoring -- --ignored --nocapture

   # Concurrent clients test
   cargo test --test e2e_changeset test_e2e_concurrent_clients -- --ignored --nocapture

   # Run all e2e tests
   cargo test --test e2e_changeset -- --ignored --nocapture
   ```

## Expected Output

```
╔═══════════════════════════════════════════════════════╗
║  E2E Changeset Test with Memory Monitoring           ║
╚═══════════════════════════════════════════════════════╝

📋 Test Configuration:
   Server URL: http://localhost:3000
   Socket.IO URL: http://localhost:3000
   Cells per mutation: 20000
   Mutation count: 3
   User ID: test_user_001
   Client ID: test_client_001

═══════════════════════════════════════════════════════
🔍 Memory Monitor Started
═══════════════════════════════════════════════════════
📊 Baseline RSS: 45.23 MB
───────────────────────────────────────────────────────

⏱️  [1s] RSS: 45.67 MB
📝 Creating document via REST API: ...
✅ Document created: ... (version: 0)
📌 Checkpoint: after_document_creation | RSS: 46.12 MB | Growth: +0.89 MB

🔌 Connecting to Socket.IO server: http://localhost:3000
✅ Connected to Socket.IO server
🎉 Socket.IO client connected successfully
📌 Checkpoint: after_socketio_connection | RSS: 48.23 MB | Growth: +3.00 MB

📥 Joining document: ...
✅ Successfully joined document: ...
   Document version: 0
📌 Checkpoint: after_join_doc | RSS: 48.45 MB | Growth: +3.22 MB

🔧 Generating 3 mutations with 20000 cells each
   Grid dimensions: 142x142 (rows x cols)
✅ Generated mutation with 20000 cells
...
📌 Checkpoint: after_mutation_generation | RSS: 52.45 MB | Growth: +7.22 MB

═══════════════════════════════════════════════════════
🚀 Starting Changeset Transmission
═══════════════════════════════════════════════════════

⏱️  Starting: changeset_1
📤 Sending changeset: doc=..., baseRev=1, mutations=1
✅ Changeset acknowledged: serverRev=2
✅ Completed: changeset_1 | serverRev=2 | Duration: 1.234s
📌 Checkpoint: after_changeset_1 | RSS: 58.34 MB | Growth: +13.11 MB

...

═══════════════════════════════════════════════════════
✅ All Changesets Sent Successfully
═══════════════════════════════════════════════════════

📊 Memory Monitor Summary
═══════════════════════════════════════════════════════
Baseline: 45.23 MB (test_start)
───────────────────────────────────────────────────────
  after_document_creation | RSS: 46.12 MB | Growth: +0.89 MB | Elapsed: 0.52s
  after_mutation_generation | RSS: 52.45 MB | Growth: +7.22 MB | Elapsed: 1.23s
  after_changeset_1 | RSS: 58.34 MB | Growth: +13.11 MB | Elapsed: 3.45s
  after_changeset_2 | RSS: 64.23 MB | Growth: +19.00 MB | Elapsed: 5.67s
  after_changeset_3 | RSS: 70.12 MB | Growth: +24.89 MB | Elapsed: 7.89s
  after_processing_complete | RSS: 68.45 MB | Growth: +23.22 MB | Elapsed: 9.91s
  after_disconnect | RSS: 65.34 MB | Growth: +20.11 MB | Elapsed: 10.02s
───────────────────────────────────────────────────────
Total Memory Growth: +20.11 MB
═══════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════╗
║  Test Completed Successfully                          ║
╚═══════════════════════════════════════════════════════╝
```

## Test Variations

### Small Test (Quick Smoke Test)
```bash
CELL_COUNT=1000 MUTATION_COUNT=1 ./run_e2e_test.sh
```
Expected duration: ~5 seconds
Expected memory growth: ~5-10 MB

### Medium Test (Default)
```bash
CELL_COUNT=20000 MUTATION_COUNT=3 ./run_e2e_test.sh
```
Expected duration: ~15-30 seconds
Expected memory growth: ~20-40 MB

### Large Test (Stress Test)
```bash
CELL_COUNT=100000 MUTATION_COUNT=10 ./run_e2e_test.sh
```
Expected duration: ~2-5 minutes
Expected memory growth: ~100-200 MB

## Troubleshooting

### Server Not Running
```
❌ Server is not responding at http://localhost:3000
```
**Solution**: Start the ot-server with `cargo run --release`

### Database Connection Error
```
Failed to create document: database connection error
```
**Solution**:
- Check PostgreSQL is running: `docker ps | grep postgres`
- Verify DATABASE_URL in .env
- Run migrations: `cd ../../migration && cargo run`

### No Ack Received
```
No ack received for changeset
```
**Solution**:
- Check server logs for errors
- Increase timeout in test (edit e2e_changeset.rs)
- Check if document was created successfully

### Memory Growth Concerns

If you see excessive memory growth:
1. Check for memory leaks using the memory monitor output
2. Compare against expected benchmarks (see E2E_TEST.md)
3. Profile with tools like valgrind or heaptrack
4. Check server logs for errors or warnings

## Next Steps

- Read [E2E_TEST.md](E2E_TEST.md) for detailed documentation
- Modify test parameters to suit your needs
- Add custom test scenarios in `tests/e2e_changeset.rs`
- Integrate with CI/CD pipeline

## Performance Tuning

For better performance in tests:

1. **Use Release Mode**:
   ```bash
   cargo run --release  # For server
   cargo test --release --test e2e_changeset -- --ignored --nocapture
   ```

2. **Adjust Database Connections**:
   ```bash
   # In .env
   DATABASE_MAX_CONNECTIONS=20
   ```

3. **Tune Redis**:
   ```bash
   # redis.conf
   maxmemory 256mb
   maxmemory-policy allkeys-lru
   ```

4. **MinIO Performance**:
   ```bash
   docker run -d --name minio \
     -p 9000:9000 \
     -e MINIO_ROOT_USER=minioadmin \
     -e MINIO_ROOT_PASSWORD=minioadmin \
     --memory 512m \
     minio/minio server /data
   ```
