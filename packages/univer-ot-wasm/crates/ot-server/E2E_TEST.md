# E2E Changeset Test with Memory Monitoring

This directory contains end-to-end tests for the OT server's Socket.IO changeset functionality, with comprehensive memory monitoring and performance tracking.

## Overview

The e2e tests simulate real-world scenarios where clients:
1. Create a new document via REST API
2. Connect to the server via Socket.IO
3. Join a document room
4. Send large changesets (mutations with thousands of cells)
5. Monitor memory usage and performance metrics throughout the process

## Test Features

### Memory Monitoring
- **Baseline Tracking**: Records initial RSS memory usage
- **Checkpoint Recording**: Captures memory at key operations
- **Periodic Logging**: Prints RSS usage every second
- **Growth Analysis**: Calculates memory growth between checkpoints
- **Summary Report**: Generates comprehensive memory analysis

### Performance Tracking
- **Operation Timing**: Measures duration of each major operation
- **Throughput Metrics**: Tracks changeset processing speed
- **Transform Latency**: Monitors OT transformation performance
- **Database Performance**: Measures persistence operations

### Stress Testing
- **Large Mutations**: Tests with 20,000+ cells per mutation
- **Multiple Changesets**: Sequential changeset processing
- **Concurrent Clients**: Multiple clients editing simultaneously
- **Memory Leaks**: Long-running tests to detect memory issues

## Test Structure

```
tests/
├── common/
│   ├── mod.rs                    # Common test utilities
│   ├── memory_monitor.rs         # Memory tracking and RSS monitoring
│   ├── socketio_client.rs        # Socket.IO test client
│   └── mutation_generator.rs     # Large mutation generation
└── e2e_changeset.rs              # Main e2e test cases
```

## Prerequisites

1. **Database**: PostgreSQL must be running and configured
2. **Redis**: Redis must be running for operation queues
3. **S3**: S3-compatible storage (MinIO or AWS S3)
4. **OT Server**: The ot-server must be running

### Environment Variables

```bash
# Database
DATABASE_URL=postgresql://user:password@localhost/univer_ot

# Redis
REDIS_URL=redis://localhost:6379

# S3 Storage
S3_ENDPOINT=http://localhost:9000
S3_REGION=us-east-1
S3_BUCKET=univer-snapshots
S3_ACCESS_KEY_ID=minioadmin
S3_SECRET_ACCESS_KEY=minioadmin

# Server Configuration
SERVER_PORT=3000
SNAPSHOT_INTERVAL=50

# Test Configuration (optional)
OT_SERVER_URL=http://localhost:3000
OT_SOCKETIO_URL=http://localhost:3000
CELL_COUNT=20000
MUTATION_COUNT=3
```

## Running Tests

### Option 1: Using the Shell Script (Recommended)

```bash
# Make the script executable
chmod +x run_e2e_test.sh

# Run with default configuration (20,000 cells, 3 mutations)
./run_e2e_test.sh

# Run with custom configuration
CELL_COUNT=50000 MUTATION_COUNT=5 ./run_e2e_test.sh

# Run the concurrent clients test
TEST_NAME=test_e2e_concurrent_clients ./run_e2e_test.sh
```

### Option 2: Using Cargo Directly

```bash
# Run the main e2e test
cargo test --test e2e_changeset test_e2e_changeset_with_memory_monitoring -- --ignored --nocapture

# Run the concurrent clients test
cargo test --test e2e_changeset test_e2e_concurrent_clients -- --ignored --nocapture

# Run all e2e tests
cargo test --test e2e_changeset -- --ignored --nocapture
```

## Test Configuration

You can customize test behavior using environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `OT_SERVER_URL` | REST API endpoint | `http://localhost:3000` |
| `OT_SOCKETIO_URL` | Socket.IO endpoint | `http://localhost:3000` |
| `CELL_COUNT` | Cells per mutation | `20000` |
| `MUTATION_COUNT` | Number of mutations | `3` |

### Examples

#### Small Test (Quick Verification)
```bash
CELL_COUNT=1000 MUTATION_COUNT=2 ./run_e2e_test.sh
```

#### Medium Test (Standard)
```bash
CELL_COUNT=20000 MUTATION_COUNT=3 ./run_e2e_test.sh
```

#### Large Test (Stress Test)
```bash
CELL_COUNT=100000 MUTATION_COUNT=10 ./run_e2e_test.sh
```

#### Memory Intensive Test
```bash
CELL_COUNT=200000 MUTATION_COUNT=20 ./run_e2e_test.sh
```

## Test Output

### Memory Monitoring Output

The test prints detailed memory information:

```
═══════════════════════════════════════════════════════
🔍 Memory Monitor Started
═══════════════════════════════════════════════════════
📊 Baseline RSS: 45.23 MB
───────────────────────────────────────────────────────

⏱️  [1s] RSS: 45.67 MB
⏱️  [2s] RSS: 48.92 MB
📌 Checkpoint: after_document_creation | RSS: 46.12 MB | Growth: +0.89 MB
⏱️  [3s] RSS: 49.23 MB
📌 Checkpoint: after_mutation_generation | RSS: 52.45 MB | Growth: +7.22 MB
⏱️  [4s] RSS: 55.67 MB
📌 Checkpoint: after_changeset_1 | RSS: 58.34 MB | Growth: +13.11 MB

═══════════════════════════════════════════════════════
📊 Memory Monitor Summary
═══════════════════════════════════════════════════════
Baseline: 45.23 MB (test_start)
───────────────────────────────────────────────────────
  after_document_creation | RSS: 46.12 MB | Growth: +0.89 MB | Elapsed: 0.52s
  after_mutation_generation | RSS: 52.45 MB | Growth: +7.22 MB | Elapsed: 1.23s
  after_changeset_1 | RSS: 58.34 MB | Growth: +13.11 MB | Elapsed: 3.45s
───────────────────────────────────────────────────────
Total Memory Growth: +13.11 MB
═══════════════════════════════════════════════════════
```

### Performance Timing Output

```
⏱️  Starting: Generate Mutations
✅ Completed: Generate Mutations | Duration: 0.234s

⏱️  Starting: changeset_1
✅ Completed: changeset_1 | serverRev=2 | Duration: 1.234s

📊 Mutation Statistics:
   Count: 3
   Total size: 1,234.56 KB
   Average size: 411.52 KB
   Mutation ID: sheet.mutation.set-range-values
```

## Test Cases

### 1. Main E2E Test (`test_e2e_changeset_with_memory_monitoring`)

**Purpose**: Comprehensive test of changeset flow with memory monitoring

**Flow**:
1. Create new document via REST API
2. Connect Socket.IO client
3. Join document room
4. Generate large mutations (20,000 cells each)
5. Send changesets sequentially
6. Monitor memory and performance at each step
7. Verify all changesets succeed
8. Disconnect and print summary

**Memory Checkpoints**:
- After document creation
- After Socket.IO connection
- After joining document
- After mutation generation
- After each changeset
- After processing complete
- After disconnect

### 2. Concurrent Clients Test (`test_e2e_concurrent_clients`)

**Purpose**: Stress test with multiple concurrent clients

**Flow**:
1. Create single document
2. Spawn multiple Socket.IO clients (default: 3)
3. Each client joins the same document
4. Clients send changesets concurrently
5. Monitor memory during concurrent operations
6. Verify all clients succeed
7. Analyze memory usage under concurrent load

**Use Cases**:
- Collaborative editing scenarios
- Race condition detection
- OT transformation under concurrency
- Database lock contention

## Troubleshooting

### Server Not Running

**Error**:
```
❌ Server is not responding at http://localhost:3000
```

**Solution**:
```bash
# Start the ot-server
cd packages/univer-ot-wasm/crates/ot-server
cargo run --release
```

### Database Connection Failed

**Error**:
```
Failed to create document: database connection error
```

**Solution**:
- Verify PostgreSQL is running
- Check `DATABASE_URL` environment variable
- Run migrations: `cd migration && cargo run`

### Socket.IO Connection Failed

**Error**:
```
Failed to connect to Socket.IO server
```

**Solution**:
- Verify server is running on correct port
- Check firewall/network settings
- Verify `OT_SOCKETIO_URL` is correct

### Memory Growth Issues

If you see excessive memory growth:

1. **Check for Memory Leaks**:
   - Look for operations that don't release memory
   - Check document actor cleanup
   - Verify S3 client connection pooling

2. **Increase Snapshot Interval**:
   ```bash
   SNAPSHOT_INTERVAL=100 cargo run --release
   ```

3. **Monitor Database Connections**:
   ```sql
   SELECT count(*) FROM pg_stat_activity;
   ```

## Performance Benchmarks

### Expected Performance (Release Build)

| Operation | Expected Duration | Notes |
|-----------|-------------------|-------|
| Document Creation | < 500ms | Including S3 snapshot |
| Socket.IO Connection | < 200ms | Initial handshake |
| Join Document | < 100ms | Room subscription |
| Mutation Generation (20k cells) | < 500ms | Pure computation |
| Changeset (20k cells) | < 2s | Transform + DB + S3 |

### Memory Usage (Release Build)

| Phase | Expected RSS | Growth |
|-------|-------------|--------|
| Baseline | ~40-50 MB | - |
| After Connection | ~50-60 MB | +10 MB |
| After 3 Changesets (20k cells each) | ~80-120 MB | +40-70 MB |
| After Disconnect | ~60-80 MB | -20-40 MB |

**Note**: Actual values depend on system configuration and data size.

## Continuous Integration

### GitHub Actions Example

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  e2e:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_DB: univer_ot
          POSTGRES_USER: postgres
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379

      minio:
        image: minio/minio
        env:
          MINIO_ROOT_USER: minioadmin
          MINIO_ROOT_PASSWORD: minioadmin
        options: >-
          --health-cmd "curl -f http://localhost:9000/minio/health/live"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 9000:9000

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run Migrations
        run: |
          cd migration
          cargo run

      - name: Start OT Server
        run: |
          cd packages/univer-ot-wasm/crates/ot-server
          cargo run --release &
          sleep 5

      - name: Run E2E Tests
        run: |
          cd packages/univer-ot-wasm/crates/ot-server
          CELL_COUNT=10000 MUTATION_COUNT=2 ./run_e2e_test.sh
```

## Advanced Usage

### Custom Test Scenarios

You can create custom test scenarios by modifying the test file:

```rust
#[tokio::test]
async fn test_custom_scenario() -> Result<()> {
    let mut monitor = MemoryMonitor::new("custom_test");

    // Your custom test logic here

    monitor.print_summary();
    Ok(())
}
```

### Memory Profiling

For detailed memory profiling, use tools like:

```bash
# Valgrind (Linux)
valgrind --tool=massif cargo test --test e2e_changeset -- --ignored

# Heaptrack (Linux)
heaptrack cargo test --test e2e_changeset -- --ignored

# Instruments (macOS)
instruments -t Leaks cargo test --test e2e_changeset -- --ignored
```

## Contributing

When adding new e2e tests:

1. Use the `MemoryMonitor` for memory tracking
2. Use `PerfTimer` for performance measurements
3. Add meaningful checkpoints at key operations
4. Include clear documentation of test purpose
5. Add environment variable configuration
6. Update this README with new test documentation

## License

Same as the parent project.
