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

import type { Buffer } from 'node:buffer';
import type { AuthService } from '../services/auth.service';
import type { AwarenessService } from '../services/awareness.service';
import type { RoomService } from '../services/room.service';
import type { AppWebSocket, AwarenessStateItem, ClientMessage, ServerMessage } from '../types';
import { logger } from '../utils/logger';

/**
 * WebSocket message handler.
 *
 * Routes incoming messages and delegates to the appropriate service.
 * This is the equivalent of socketio.rs's on_connect + event handlers.
 */
export class WebSocketHandler {
    constructor(
        private readonly _roomService: RoomService,
        private readonly _awarenessService: AwarenessService,
        private readonly _authService: AuthService
    ) {}

    /**
     * Handle an incoming WebSocket message.
     */
    async handleMessage(ws: AppWebSocket, raw: string | Buffer): Promise<void> {
        let msg: ClientMessage;
        try {
            msg = JSON.parse(typeof raw === 'string' ? raw : raw.toString()) as ClientMessage;
        } catch {
            ws.send(JSON.stringify({ type: 'error', message: 'Invalid JSON' }));
            return;
        }

        switch (msg.type) {
            case 'join_doc':
                await this._handleJoinDoc(ws, msg.docId, msg.requestId);
                break;
            case 'leave_doc':
                await this._handleLeaveDoc(ws, msg.docId);
                break;
            case 'presence_update':
                await this._handlePresenceUpdate(ws, msg.docId, msg.user, msg.selectionParams);
                break;
            case 'awareness_init':
                await this._handleAwarenessInit(ws, msg.docId, msg.requestId);
                break;
            case 'pong':
                ws.data.lastPong = Date.now();
                break;
            default:
                ws.send(JSON.stringify({ type: 'error', message: 'Unknown message type' }));
        }
    }

    /**
     * Handle join_doc: verify permissions, join room, send ack.
     *
     * Unlike the Rust socketio handler, we do NOT return the document version.
     * The client should call GET /api/documents/{docId} on ot-server to get
     * the version independently. This keeps ws-gateway stateless w.r.t. documents.
     */
    private async _handleJoinDoc(
        ws: AppWebSocket,
        docId: string,
        requestId: string
    ): Promise<void> {
        try {
            // Check readable permission
            const perms = await this._authService.checkDocumentPermission(
                ws.data.userId,
                docId,
                ws.data.accessToken
            );

            if (!perms.readable) {
                ws.send(
                    JSON.stringify({
                        type: 'join_doc_ack',
                        requestId,
                        status: 'error',
                        message: `Permission denied: user ${ws.data.userId} cannot read document ${docId}`,
                    })
                );
                return;
            }

            this._roomService.join(docId, ws);

            ws.send(
                JSON.stringify({
                    type: 'join_doc_ack',
                    requestId,
                    status: 'ok',
                })
            );
        } catch (e) {
            logger.error(`join_doc failed for doc ${docId}`, e);
            ws.send(
                JSON.stringify({
                    type: 'join_doc_ack',
                    requestId,
                    status: 'error',
                    message: e instanceof Error ? e.message : 'Unknown error',
                })
            );
        }
    }

    /**
     * Handle leave_doc: leave room, clean up awareness.
     */
    private async _handleLeaveDoc(ws: AppWebSocket, docId: string): Promise<void> {
        this._roomService.leave(docId, ws);
        await this._awarenessService.removeBySocket(docId, ws.data.clientId);
        logger.info(`Client ${ws.data.clientId} left doc ${docId}`);
    }

    /**
     * Handle presence_update: upsert awareness, broadcast to room (excluding sender).
     */
    private async _handlePresenceUpdate(
        ws: AppWebSocket,
        docId: string,
        user: { id: string; name: string },
        selectionParams?: unknown
    ): Promise<void> {
        const clientId = ws.data.clientId;

        const awarenessItem: AwarenessStateItem = {
            clientID: clientId,
            id: user.id,
            name: user.name,
            selectionParams: selectionParams ?? {
                unitId: '',
                subUnitId: '',
                selections: [],
            },
        };

        await this._awarenessService.upsertState(docId, clientId, awarenessItem);

        // Broadcast to room, excluding sender
        const broadcastMsg: ServerMessage = {
            type: 'presence_update',
            clientID: clientId,
            id: user.id,
            name: user.name,
            selectionParams,
        };
        this._roomService.broadcastToRoom(docId, broadcastMsg, ws);
    }

    /**
     * Handle awareness_init: return current awareness states for the document.
     */
    private async _handleAwarenessInit(
        ws: AppWebSocket,
        docId: string,
        requestId: string
    ): Promise<void> {
        try {
            const states = await this._awarenessService.getState(docId);
            ws.send(
                JSON.stringify({
                    type: 'awareness_init_ack',
                    requestId,
                    status: 'ok',
                    states,
                })
            );
        } catch (e) {
            logger.error(`awareness_init failed for doc ${docId}`, e);
            ws.send(
                JSON.stringify({
                    type: 'awareness_init_ack',
                    requestId,
                    status: 'error',
                    states: [],
                })
            );
        }
    }

    /**
     * Handle WebSocket close: leave all rooms, clean up awareness.
     */
    async handleClose(ws: AppWebSocket): Promise<void> {
        const clientId = ws.data.clientId;
        logger.info(`Client ${clientId} (user ${ws.data.userId}) disconnecting`);

        const docIds = this._roomService.leaveAll(ws);

        // Async cleanup of awareness state (non-blocking)
        for (const docId of docIds) {
            try {
                await this._awarenessService.removeBySocket(docId, clientId);
            } catch (e) {
                logger.warn(`Failed to clean awareness for doc ${docId}, client ${clientId}`, e);
            }
        }

        logger.info(`Client ${clientId} disconnected, left ${docIds.length} rooms`);
    }
}
