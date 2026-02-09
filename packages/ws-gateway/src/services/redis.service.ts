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

import type { RedisClientType } from 'redis';
import type { RedisBroadcastMessage } from '../types';
import { createClient } from 'redis';
import { logger } from '../utils/logger';
import { getMetrics } from './metrics.service';

const BROADCAST_CHANNEL = 'ws:broadcast';

export class RedisService {
    /** Command connection for read/write operations */
    private _cmdClient: RedisClientType;
    /** Subscriber connection for Pub/Sub */
    private _subClient: RedisClientType;
    /** Callback when a broadcast message is received */
    private _onBroadcast: ((msg: RedisBroadcastMessage) => void) | null = null;

    constructor(redisUrl: string) {
        this._cmdClient = createClient({ url: redisUrl }) as RedisClientType;
        this._subClient = this._cmdClient.duplicate() as RedisClientType;

        this._cmdClient.on('error', (err) => logger.error('Redis cmd error', err));
        this._subClient.on('error', (err) => logger.error('Redis sub error', err));
    }

    async connect(): Promise<void> {
        await Promise.all([this._cmdClient.connect(), this._subClient.connect()]);

        // Subscribe to broadcast channel
        await this._subClient.subscribe(BROADCAST_CHANNEL, (message) => {
            try {
                const parsed = JSON.parse(message) as RedisBroadcastMessage;
                this._onBroadcast?.(parsed);
            } catch (e) {
                logger.error('Failed to parse broadcast message', e);
            }
        });

        logger.info(`Redis connected, subscribed to ${BROADCAST_CHANNEL}`);
    }

    onBroadcast(handler: (msg: RedisBroadcastMessage) => void): void {
        this._onBroadcast = handler;
    }

    // ─── Awareness Redis Operations ─────────────────────────

    async getAwareness(docId: string): Promise<string | null> {
        const key = `awareness:doc:${docId}`;
        return getMetrics().measureAsync(
            () => this._cmdClient.get(key),
            (duration) => getMetrics().recordRedisOperationDuration(duration, 'get_awareness')
        );
    }

    async setAwareness(docId: string, payload: string, ttlSeconds: number): Promise<void> {
        const key = `awareness:doc:${docId}`;
        return getMetrics().measureAsync(
            () => this._cmdClient.setEx(key, ttlSeconds, payload),
            (duration) => getMetrics().recordRedisOperationDuration(duration, 'set_awareness')
        );
    }

    async deleteAwareness(docId: string): Promise<void> {
        const key = `awareness:doc:${docId}`;
        return getMetrics().measureAsync(
            () => this._cmdClient.del(key),
            (duration) => getMetrics().recordRedisOperationDuration(duration, 'delete_awareness')
        );
    }

    async disconnect(): Promise<void> {
        await this._subClient.unsubscribe(BROADCAST_CHANNEL);
        await Promise.all([this._cmdClient.quit(), this._subClient.quit()]);
    }
}
