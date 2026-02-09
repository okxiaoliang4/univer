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

import type { AwarenessStateItem } from '../types';
import type { RedisService } from './redis.service';
import { logger } from '../utils/logger';
import { getMetrics } from './metrics.service';

/**
 * Awareness state management service.
 *
 * Ported from Rust `awareness.rs`. Maintains in-memory state with Redis
 * persistence for cross-instance consistency.
 *
 * Memory layout:
 * - `_memory`: Map<docId, Map<clientId, AwarenessStateItem>>
 * - `_socketMap`: Map<docId, Map<socketId, clientId>> for cleanup on disconnect
 */
export class AwarenessService {
    /** docId → (clientId → AwarenessStateItem) */
    private _memory = new Map<string, Map<string, AwarenessStateItem>>();
    /** docId → (socketId → clientId) for disconnect cleanup */
    private _socketMap = new Map<string, Map<string, string>>();

    constructor(
        private readonly _redis: RedisService,
        private readonly _ttlSeconds: number
    ) {}

    async getState(docId: string): Promise<AwarenessStateItem[]> {
        // Try Redis first
        try {
            const raw = await this._redis.getAwareness(docId);
            if (raw) {
                const snapshot = JSON.parse(raw) as { states: AwarenessStateItem[] };
                const stateMap = new Map<string, AwarenessStateItem>();
                for (const item of snapshot.states) {
                    stateMap.set(item.clientID, item);
                }
                this._memory.set(docId, stateMap);
                return snapshot.states;
            }
        } catch (e) {
            logger.warn(`Failed to read awareness from Redis for doc ${docId}`, e);
        }

        // Fall back to memory
        const mem = this._memory.get(docId);
        return mem ? [...mem.values()] : [];
    }

    async upsertState(
        docId: string,
        socketId: string,
        state: AwarenessStateItem
    ): Promise<void> {
        const clientId = state.clientID;

        // Update memory
        let docState = this._memory.get(docId);
        if (!docState) {
            docState = new Map();
            this._memory.set(docId, docState);
        }
        docState.set(clientId, state);

        // Update socket mapping
        let socketDocMap = this._socketMap.get(docId);
        if (!socketDocMap) {
            socketDocMap = new Map();
            this._socketMap.set(docId, socketDocMap);
        }
        socketDocMap.set(socketId, clientId);

        // Update metrics
        this._updateMetrics();

        // Persist to Redis
        await this._persistDoc(docId);
    }

    async removeBySocket(docId: string, socketId: string): Promise<void> {
        const socketDocMap = this._socketMap.get(docId);
        if (!socketDocMap) return;

        const clientId = socketDocMap.get(socketId);
        if (!clientId) return;

        socketDocMap.delete(socketId);
        if (socketDocMap.size === 0) {
            this._socketMap.delete(docId);
        }

        // Remove client from memory
        const docState = this._memory.get(docId);
        if (docState) {
            docState.delete(clientId);
            if (docState.size === 0) {
                this._memory.delete(docId);
            }
        }

        // Update metrics
        this._updateMetrics();

        await this._persistDoc(docId);
    }

    private async _persistDoc(docId: string): Promise<void> {
        try {
            const docState = this._memory.get(docId);
            const states = docState ? [...docState.values()] : [];
            const payload = JSON.stringify({ states });
            await this._redis.setAwareness(docId, payload, this._ttlSeconds);
        } catch (e) {
            logger.warn(`Failed to persist awareness for doc ${docId}`, e);
        }
    }

    private _updateMetrics(): void {
        // Count total awareness states across all documents
        let totalStates = 0;
        for (const docState of this._memory.values()) {
            totalStates += docState.size;
        }
        getMetrics().setAwarenessStates(totalStates);
    }
}
