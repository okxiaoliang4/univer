# WS-Gateway Monitoring

## Metrics Endpoint

The ws-gateway exposes Prometheus-compatible metrics on a dedicated port:

```
http://localhost:9464/metrics
```

The service uses two ports:
- **Port 8080** (WebSocket server):
  - WebSocket connections: `ws://localhost:8080/ws?token=<token>`
  - Health check: `http://localhost:8080/health`
- **Port 9464** (Metrics server):
  - Metrics: `http://localhost:9464/metrics`

## Available Metrics

### Connection Metrics

- **`ws_connections_total`** (Gauge)
  - Current number of active WebSocket connections
  - Labels: none

- **`ws_rooms_total`** (Gauge)
  - Current number of active rooms (documents)
  - Labels: none

- **`ws_awareness_states_total`** (Gauge)
  - Current number of awareness states in memory
  - Labels: none

### Message Metrics

- **`ws_messages_received_total`** (Counter)
  - Total number of WebSocket messages received
  - Labels: `message_type` (join_doc, leave_doc, presence_update, awareness_init, pong)

- **`ws_messages_sent_total`** (Counter)
  - Total number of WebSocket messages sent
  - Labels: `message_type` (join_doc_ack, awareness_init_ack, changeset_pushed, presence_update, ping)

### Latency Metrics

- **`ws_message_processing_duration_seconds`** (Histogram)
  - Time taken to process WebSocket messages
  - Labels: `message_type`
  - Buckets: Default histogram buckets

- **`ws_redis_operation_duration_seconds`** (Histogram)
  - Time taken for Redis operations
  - Labels: `operation` (get_awareness, set_awareness, delete_awareness)
  - Buckets: Default histogram buckets

### Error Metrics

- **`ws_errors_total`** (Counter)
  - Total number of errors
  - Labels: `error_type` (websocket_upgrade_failed, message_handling_error, close_handling_error)

- **`ws_auth_failures_total`** (Counter)
  - Total number of authentication failures
  - Labels: `reason` (token_verification_failed)

## Example Prometheus Configuration

Add this to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'ws-gateway'
    static_configs:
      - targets: ['localhost:9464']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

## Example Grafana Queries

### Connection Rate
```promql
rate(ws_connections_total[5m])
```

### Message Processing Latency (p95)
```promql
histogram_quantile(0.95, rate(ws_message_processing_duration_seconds_bucket[5m]))
```

### Messages Received Rate by Type
```promql
sum by (message_type) (rate(ws_messages_received_total[5m]))
```

### Redis Operation Latency (p99)
```promql
histogram_quantile(0.99, rate(ws_redis_operation_duration_seconds_bucket[5m]))
```

### Error Rate
```promql
rate(ws_errors_total[5m])
```

### Active Rooms
```promql
ws_rooms_total
```

## Docker Compose Setup

The service exposes two ports:

```yaml
ws-gateway:
  ports:
    - '8080:8080'  # WebSocket server + Health check
    - '9464:9464'  # Metrics endpoint
  environment:
    PORT: 8080
    METRICS_PORT: 9464
```

Access metrics via:
```bash
curl http://localhost:9464/metrics
```

## Health Monitoring Dashboard

Key metrics to monitor:

1. **Connection Health**
   - `ws_connections_total` - Should be stable or gradually increasing
   - Alert if drops suddenly

2. **Message Processing**
   - `ws_message_processing_duration_seconds` - Should be < 100ms p95
   - Alert if p95 > 500ms

3. **Redis Performance**
   - `ws_redis_operation_duration_seconds` - Should be < 10ms p95
   - Alert if p95 > 100ms

4. **Error Rate**
   - `rate(ws_errors_total[5m])` - Should be near zero
   - Alert if > 0.1 errors/sec

5. **Room Utilization**
   - `ws_rooms_total` - Track document collaboration activity
   - `ws_awareness_states_total` - Track active users

## OpenTelemetry Integration

The metrics service uses OpenTelemetry SDK with Prometheus exporter. To add additional exporters (e.g., OTLP):

```typescript
import { OTLPMetricExporter } from '@opentelemetry/exporter-metrics-otlp-http';
import { PeriodicExportingMetricReader } from '@opentelemetry/sdk-metrics';

const otlpExporter = new OTLPMetricExporter({
  url: 'http://otel-collector:4318/v1/metrics',
});

const meterProvider = new MeterProvider({
  resource,
  readers: [
    prometheusExporter,
    new PeriodicExportingMetricReader({
      exporter: otlpExporter,
      exportIntervalMillis: 60000,
    }),
  ],
});
```
