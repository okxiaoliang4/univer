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

import type { AppWebSocket, ServerMessage } from '../types';
import { logger } from '../utils/logger';
import { getMetrics } from './metrics.service';

/**
 * In-memory room management service.
 *
 * Tracks which WebSocket connections belong to which document rooms.
 * Provides efficient broadcast to all members of a room, optionally
 * excluding the sender.
 */
export class RoomService {
    /** docId → Set of WebSocket connections in the room */
    private _rooms = new Map<string, Set<AppWebSocket>>();
    /** WebSocket → Set of docIds that connection has joined */
    private _wsRooms = new Map<AppWebSocket, Set<string>>();

    join(docId: string, ws: AppWebSocket): void {
        // Add ws to room
        let room = this._rooms.get(docId);
        if (!room) {
            room = new Set();
            this._rooms.set(docId, room);
        }
        room.add(ws);

        // Track reverse mapping
        let rooms = this._wsRooms.get(ws);
        if (!rooms) {
            rooms = new Set();
            this._wsRooms.set(ws, rooms);
        }
        rooms.add(docId);

        // Update metrics
        getMetrics().setActiveRooms(this._rooms.size);

        logger.info(`Client ${ws.data.clientId} joined room ${docId} (${room.size} members)`);
    }

    leave(docId: string, ws: AppWebSocket): void {
        const room = this._rooms.get(docId);
        if (room) {
            room.delete(ws);
            if (room.size === 0) {
                this._rooms.delete(docId);
            }
            logger.info(`Client ${ws.data.clientId} left room ${docId} (${room.size} remaining)`);
        }

        const rooms = this._wsRooms.get(ws);
        if (rooms) {
            rooms.delete(docId);
            if (rooms.size === 0) {
                this._wsRooms.delete(ws);
            }
        }

        // Update metrics
        getMetrics().setActiveRooms(this._rooms.size);
    }

    /**
     * Remove a WebSocket from all rooms it has joined.
     * Returns the list of docIds that were left.
     */
    leaveAll(ws: AppWebSocket): string[] {
        const rooms = this._wsRooms.get(ws);
        if (!rooms) return [];

        const docIds = [...rooms];
        for (const docId of docIds) {
            this.leave(docId, ws);
        }
        this._wsRooms.delete(ws);
        return docIds;
    }

    /**
     * Broadcast a message to all members of a room.
     * @param exclude - optionally exclude one ws (e.g. the sender)
     */
    broadcastToRoom(docId: string, msg: ServerMessage, exclude?: AppWebSocket): void {
        const room = this._rooms.get(docId);
        if (!room) return;

        const payload = JSON.stringify(msg);
        let sentCount = 0;
        for (const ws of room) {
            if (ws !== exclude) {
                try {
                    ws.send(payload);
                    sentCount++;
                } catch {
                    // Connection may have been dropped; ignore
                }
            }
        }

        // Record sent messages
        if (sentCount > 0) {
            getMetrics().recordMessageSent(msg.type);
        }
    }

    getRoomMembers(docId: string): Set<AppWebSocket> | undefined {
        return this._rooms.get(docId);
    }

    getRoomSize(docId: string): number {
        return this._rooms.get(docId)?.size ?? 0;
    }

    getRoomsForWs(ws: AppWebSocket): string[] {
        return [...(this._wsRooms.get(ws) ?? [])];
    }
}
