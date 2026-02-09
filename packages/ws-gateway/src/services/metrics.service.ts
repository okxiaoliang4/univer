/**
 * Copyright 2023-present DreamNum Co., Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import type { Counter, Histogram, Meter, ObservableGauge } from '@opentelemetry/api';
import type { Server } from 'node:http';
import http from 'node:http';
import { PrometheusExporter } from '@opentelemetry/exporter-prometheus';
import { resourceFromAttributes } from '@opentelemetry/resources';
import { MeterProvider } from '@opentelemetry/sdk-metrics';
import { ATTR_SERVICE_NAME, ATTR_SERVICE_VERSION } from '@opentelemetry/semantic-conventions';
import { logger } from '../utils/logger';

/**
 * Metrics service for collecting and exposing metrics
 *
 * Key metrics:
 * - ws_connections_total: Total active WebSocket connections
 * - ws_rooms_total: Total active rooms
 * - ws_awareness_states_total: Total awareness states in memory
 * - ws_messages_received_total: Total messages received (by type)
 * - ws_messages_sent_total: Total messages sent (by type)
 * - ws_message_processing_duration_seconds: Message processing latency
 * - ws_redis_operation_duration_seconds: Redis operation latency
 * - ws_errors_total: Total errors (by type)
 * - ws_auth_failures_total: Total authentication failures
 *
 * The metrics are exposed via HTTP endpoint /metrics (same port as WebSocket server)
 */
export class MetricsService {
    private _meter: Meter;
    private _exporter: PrometheusExporter;
    private _server: Server | null = null;

    // Counters
    private _messagesReceived!: Counter;
    private _messagesSent!: Counter;
    private _errorsTotal!: Counter;
    private _authFailures!: Counter;

    // Histograms
    private _messageProcessingDuration!: Histogram;
    private _redisOperationDuration!: Histogram;

    // Gauges (observable)
    private _connectionsGauge!: ObservableGauge;
    private _roomsGauge!: ObservableGauge;
    private _awarenessStatesGauge!: ObservableGauge;

    // Internal state tracking
    private _activeConnections = 0;
    private _activeRooms = 0;
    private _awarenessStates = 0;

    constructor() {
        // Create Prometheus exporter
        // We'll start the HTTP server separately via startServer()
        this._exporter = new PrometheusExporter();

        // Create meter provider with resource information
        const resource = resourceFromAttributes({
            [ATTR_SERVICE_NAME]: 'ws-gateway',
            [ATTR_SERVICE_VERSION]: '0.15.0',
        });

        const meterProvider = new MeterProvider({
            resource,
            readers: [this._exporter],
        });

        this._meter = meterProvider.getMeter('ws-gateway');

        // Initialize metrics
        this._initCounters();
        this._initHistograms();
        this._initGauges();

        logger.info('MetricsService initialized');
    }

    private _initCounters(): void {
        // Message counters
        this._messagesReceived = this._meter.createCounter('ws_messages_received_total', {
            description: 'Total number of WebSocket messages received',
            unit: '1',
        });

        this._messagesSent = this._meter.createCounter('ws_messages_sent_total', {
            description: 'Total number of WebSocket messages sent',
            unit: '1',
        });

        // Error counters
        this._errorsTotal = this._meter.createCounter('ws_errors_total', {
            description: 'Total number of errors',
            unit: '1',
        });

        this._authFailures = this._meter.createCounter('ws_auth_failures_total', {
            description: 'Total number of authentication failures',
            unit: '1',
        });
    }

    private _initHistograms(): void {
        // Latency histograms
        this._messageProcessingDuration = this._meter.createHistogram('ws_message_processing_duration_seconds', {
            description: 'Time taken to process WebSocket messages',
            unit: 's',
        });

        this._redisOperationDuration = this._meter.createHistogram('ws_redis_operation_duration_seconds', {
            description: 'Time taken for Redis operations',
            unit: 's',
        });
    }

    private _initGauges(): void {
        // Connection gauge
        this._connectionsGauge = this._meter.createObservableGauge('ws_connections_total', {
            description: 'Current number of active WebSocket connections',
            unit: '1',
        });
        this._connectionsGauge.addCallback((result) => {
            result.observe(this._activeConnections);
        });

        // Rooms gauge
        this._roomsGauge = this._meter.createObservableGauge('ws_rooms_total', {
            description: 'Current number of active rooms',
            unit: '1',
        });
        this._roomsGauge.addCallback((result) => {
            result.observe(this._activeRooms);
        });

        // Awareness states gauge
        this._awarenessStatesGauge = this._meter.createObservableGauge('ws_awareness_states_total', {
            description: 'Current number of awareness states in memory',
            unit: '1',
        });
        this._awarenessStatesGauge.addCallback((result) => {
            result.observe(this._awarenessStates);
        });
    }

    // ─── Public API for recording metrics ───────────────────────

    /**
     * Record a message received
     */
    recordMessageReceived(type: string): void {
        this._messagesReceived.add(1, { message_type: type });
    }

    /**
     * Record a message sent
     */
    recordMessageSent(type: string): void {
        this._messagesSent.add(1, { message_type: type });
    }

    /**
     * Record message processing duration
     */
    recordMessageProcessingDuration(durationSeconds: number, type: string): void {
        this._messageProcessingDuration.record(durationSeconds, { message_type: type });
    }

    /**
     * Record Redis operation duration
     */
    recordRedisOperationDuration(durationSeconds: number, operation: string): void {
        this._redisOperationDuration.record(durationSeconds, { operation });
    }

    /**
     * Record an error
     */
    recordError(errorType: string): void {
        this._errorsTotal.add(1, { error_type: errorType });
    }

    /**
     * Record an authentication failure
     */
    recordAuthFailure(reason: string): void {
        this._authFailures.add(1, { reason });
    }

    /**
     * Update active connections count
     */
    setActiveConnections(count: number): void {
        this._activeConnections = count;
    }

    /**
     * Increment active connections
     */
    incrementConnections(): void {
        this._activeConnections++;
    }

    /**
     * Decrement active connections
     */
    decrementConnections(): void {
        this._activeConnections = Math.max(0, this._activeConnections - 1);
    }

    /**
     * Update active rooms count
     */
    setActiveRooms(count: number): void {
        this._activeRooms = count;
    }

    /**
     * Update awareness states count
     */
    setAwarenessStates(count: number): void {
        this._awarenessStates = count;
    }

    /**
     * Start the metrics HTTP server on the specified port
     */
    async startServer(port: number): Promise<void> {
        return new Promise((resolve, reject) => {
            try {
                // Use Node.js http module to create server
                // This is compatible with PrometheusExporter's getMetricsRequestHandler

                this._server = http.createServer(
                    (req, res) => {
                        if (req.url === '/metrics') {
                            this._exporter.getMetricsRequestHandler(req, res);
                        }
                    }
                );

                this._server.listen(port, () => {
                    logger.info(`Metrics server listening on port ${port}`);
                    resolve();
                });

                this._server.on('error', (err: Error) => {
                    logger.error('Metrics server error', err);
                    reject(err);
                });
            } catch (err) {
                logger.error('Failed to start metrics server', err);
                reject(err);
            }
        });
    }

    /**
     * Stop the metrics HTTP server
     */
    async stopServer(): Promise<void> {
        if (this._server) {
            return new Promise((resolve) => {
                this._server!.close(() => {
                    logger.info('Metrics server stopped');
                    resolve();
                });
            });
        }
    }

    /**
     * Helper to measure function execution time
     */
    async measureAsync<T>(
        fn: () => Promise<T>,
        recordFn: (duration: number) => void
    ): Promise<T> {
        const start = performance.now();
        try {
            return await fn();
        } finally {
            const duration = (performance.now() - start) / 1000; // Convert to seconds
            recordFn(duration);
        }
    }

    /**
     * Helper to measure sync function execution time
     */
    measure<T>(
        fn: () => T,
        recordFn: (duration: number) => void
    ): T {
        const start = performance.now();
        try {
            return fn();
        } finally {
            const duration = (performance.now() - start) / 1000; // Convert to seconds
            recordFn(duration);
        }
    }
}

// Singleton instance
let metricsService: MetricsService | null = null;

/**
 * Initialize metrics service
 */
export function initMetrics(): MetricsService {
    if (!metricsService) {
        metricsService = new MetricsService();
    }
    return metricsService;
}

/**
 * Get metrics service instance
 */
export function getMetrics(): MetricsService {
    if (!metricsService) {
        throw new Error('MetricsService not initialized. Call initMetrics() first.');
    }
    return metricsService;
}
