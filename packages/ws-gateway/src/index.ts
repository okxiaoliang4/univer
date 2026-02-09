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

import process from 'node:process';
import { loadConfig } from './config';
import { createServer } from './server';
import { AuthService } from './services/auth.service';
import { AwarenessService } from './services/awareness.service';
import { GrpcClientService } from './services/grpc-client.service';
import { initMetrics } from './services/metrics.service';
import { RedisService } from './services/redis.service';
import { RoomService } from './services/room.service';
import { logger } from './utils/logger';

async function main() {
    logger.info('Starting ws-gateway...');

    // Load configuration
    const config = loadConfig();
    logger.info(`Config: port=${config.port}, redis=${config.redisUrl}, etcd=${config.etcdEndpoints.join(',')}`);

    // Initialize metrics
    const metricsService = initMetrics();
    await metricsService.startServer(config.metricsPort);
    logger.info('Metrics service initialized');

    // Initialize Redis
    const redisService = new RedisService(config.redisUrl);
    await redisService.connect();
    logger.info('Redis connected');

    // Initialize gRPC client (etcd service discovery)
    const grpcClient = new GrpcClientService(
        config.etcdEndpoints,
        config.userRpcPrefix,
        config.documentRpcPrefix
    );
    logger.info('gRPC client initialized');

    // Initialize auth service
    const authService = new AuthService(
        grpcClient,
        config.skipTokenVerification,
        config.skipPermissionCheck
    );
    logger.info('Auth service initialized');

    // Initialize room and awareness services
    const roomService = new RoomService();
    const awarenessService = new AwarenessService(redisService, config.awarenessTtlSeconds);
    logger.info('Room and awareness services initialized');

    // Start the server
    const server = await createServer(
        config,
        authService,
        roomService,
        awarenessService,
        redisService
    );

    logger.info(`ws-gateway listening on port ${config.port}`);
    logger.info(`WebSocket endpoint: ws://0.0.0.0:${config.port}/ws?token=<token>`);
    logger.info(`Health check: http://0.0.0.0:${config.port}/health`);
    logger.info(`Metrics endpoint: http://0.0.0.0:${config.metricsPort}/metrics`);

    // Graceful shutdown
    process.on('SIGTERM', async () => {
        logger.info('SIGTERM received, shutting down...');
        server.stop();
        await metricsService.stopServer();
        await redisService.disconnect();
        await grpcClient.close();
        logger.info('Shutdown complete');
        process.exit(0);
    });
}

main().catch((e) => {
    logger.error('Fatal error:', e);
    process.exit(1);
});
