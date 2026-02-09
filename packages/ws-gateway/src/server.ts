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

import type { Server } from 'bun';
import type { IConfig } from './config';
import type { AuthService } from './services/auth.service';
import type { AwarenessService } from './services/awareness.service';
import type { RedisService } from './services/redis.service';
import type { RoomService } from './services/room.service';
import type { IWsData, RedisBroadcastMessage } from './types';
import process from 'node:process';
import { WebSocketHandler } from './handlers/websocket.handler';
import { getMetrics } from './services/metrics.service';
import { logger } from './utils/logger';

/**
 * Create and start the Bun WebSocket server.
 *
 * The server handles:
 * - HTTP upgrade with token authentication (from query `?token=xxx`)
 * - WebSocket message routing via WebSocketHandler
 * - Ping/pong heartbeat for connection liveness
 * - Redis Pub/Sub consumption for broadcasting changeset_pushed
 */
// eslint-disable-next-line max-lines-per-function
export async function createServer(
    config: IConfig,
    authService: AuthService,
    roomService: RoomService,
    awarenessService: AwarenessService,
    redisService: RedisService
): Promise<Server<IWsData>> {
    const wsHandler = new WebSocketHandler(roomService, awarenessService, authService);

    // Listen for Redis broadcast messages and forward to rooms
    redisService.onBroadcast((msg: RedisBroadcastMessage) => {
        if (msg.type === 'changeset_pushed') {
            roomService.broadcastToRoom(msg.docId, {
                type: 'changeset_pushed',
                docId: msg.docId,
                serverRev: msg.serverRev,
                userId: msg.userId,
            });
        }
    });

    let clientCounter = 0;

    const server = Bun.serve<IWsData>({
        port: config.port,

        async fetch(req, server) {
            const url = new URL(req.url);

            // Health check endpoint
            if (url.pathname === '/health') {
                return new Response('OK', { status: 200 });
            }

            // WebSocket upgrade
            if (url.pathname === '/ws' || url.pathname === '/') {
                const token = url.searchParams.get('token');
                if (!token) {
                    return new Response('Missing token', { status: 401 });
                }

                try {
                    const userInfo = await authService.verifyToken(token);
                    const clientId = `ws-${++clientCounter}-${Date.now()}`;

                    const wsData: IWsData = {
                        userId: userInfo.uid,
                        email: userInfo.email,
                        accessToken: token,
                        clientId,
                        lastPong: Date.now(),
                    };

                    const success = server.upgrade(req, { data: wsData });
                    if (!success) {
                        getMetrics().recordError('websocket_upgrade_failed');
                        return new Response('WebSocket upgrade failed', { status: 500 });
                    }
                    // Bun returns undefined on successful upgrade
                    return undefined as unknown as Response;
                } catch (e) {
                    logger.warn(`Auth failed during WS upgrade: ${e}`);
                    getMetrics().recordAuthFailure('token_verification_failed');
                    return new Response('Authentication failed', { status: 401 });
                }
            }

            return new Response('Not Found', { status: 404 });
        },

        websocket: {
            open(ws) {
                logger.info(
                    `WebSocket opened: clientId=${ws.data.clientId}, userId=${ws.data.userId}`
                );
                getMetrics().incrementConnections();
            },

            async message(ws, message) {
                const startTime = performance.now();
                let messageType = 'unknown';
                try {
                    // Parse message type for metrics
                    if (typeof message === 'string') {
                        try {
                            const parsed = JSON.parse(message);
                            messageType = parsed.type || 'unknown';
                        } catch {
                            // Ignore parse errors for metrics
                        }
                    }

                    getMetrics().recordMessageReceived(messageType);
                    await wsHandler.handleMessage(ws, message);

                    const duration = (performance.now() - startTime) / 1000;
                    getMetrics().recordMessageProcessingDuration(duration, messageType);
                } catch (e) {
                    logger.error(`Error handling message from ${ws.data.clientId}`, e);
                    getMetrics().recordError('message_handling_error');
                }
            },

            async close(ws, code, reason) {
                logger.info(
                    `WebSocket closed: clientId=${ws.data.clientId}, code=${code}, reason=${reason}`
                );
                getMetrics().decrementConnections();
                try {
                    await wsHandler.handleClose(ws);
                } catch (e) {
                    logger.error(`Error handling close for ${ws.data.clientId}`, e);
                    getMetrics().recordError('close_handling_error');
                }
            },

            // Bun handles ping/pong at the protocol level automatically.
            // We add application-level ping for more reliable detection.
            perMessageDeflate: false,
            maxPayloadLength: 16 * 1024 * 1024, // 16MB max message size
            idleTimeout: 120, // seconds — Bun will close idle connections
        },
    });

    // Application-level ping interval
    const pingInterval = setInterval(() => {
        // Bun doesn't expose a global list of websockets easily,
        // so we rely on the idleTimeout and Bun's built-in ping/pong.
        // For more control, we can track connections in RoomService.
    }, config.pingIntervalMs);

    // Cleanup on process exit
    process.on('SIGINT', () => {
        clearInterval(pingInterval);
        server.stop();
    });

    return server;
}
