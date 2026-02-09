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

import type { ServerWebSocket } from 'bun';

/**
 * Data attached to each WebSocket connection via `ws.data`
 */
export interface IWsData {
    userId: string;
    email: string;
    accessToken: string;
    /** Unique client ID for this connection */
    clientId: string;
    /** Timestamp of last pong received */
    lastPong: number;
}

export type AppWebSocket = ServerWebSocket<IWsData>;

// ─── Client → Server Messages ───────────────────────────────

export type ClientMessage =
    | { type: 'join_doc'; docId: string; requestId: string }
    | { type: 'leave_doc'; docId: string }
    | {
        type: 'presence_update';
        docId: string;
        user: { id: string; name: string };
        selectionParams?: unknown;
    }
    | { type: 'awareness_init'; docId: string; requestId: string }
    | { type: 'pong' };

// ─── Server → Client Messages ───────────────────────────────

export type ServerMessage =
    | {
        type: 'join_doc_ack';
        requestId: string;
        status: string;
        message?: string;
    }
    | {
        type: 'awareness_init_ack';
        requestId: string;
        status: string;
        states: AwarenessStateItem[];
    }
    | {
        type: 'changeset_pushed';
        docId: string;
        serverRev: number;
        userId: string;
    }
    | {
        type: 'presence_update';
        clientID: string;
        id: string;
        name: string;
        selectionParams?: unknown;
    }
    | { type: 'ping' }
    | { type: 'error'; message: string };

// ─── Awareness Types ────────────────────────────────────────

export interface AwarenessStateItem {
    clientID: string;
    id: string;
    name: string;
    selectionParams: unknown;
}

// ─── Redis Broadcast Types ──────────────────────────────────

export interface RedisBroadcastMessage {
    type: 'changeset_pushed';
    docId: string;
    serverRev: number;
    userId: string;
}
