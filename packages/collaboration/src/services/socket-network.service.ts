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

import type { Nullable } from '@univerjs/core';
import type { Socket } from 'socket.io-client';
import type {
    IChangesetAck,
    IChangesetPushed,
    IChangesetRequest,
    IFetchOpsResult,
    IJoinDocAck,
    IUserAwareness,
    NetworkConnectionStatus,
} from '../common/types';
import type { ICollaborationConfig } from '../controller/config.schema';
import type { IAwarenessInitResult, INetworkService } from './network.service';
import {
    Disposable,
    IConfigService,
    ILogService,
    Inject,
} from '@univerjs/core';
import { BehaviorSubject, Subject } from 'rxjs';
import { io } from 'socket.io-client';
import {
    COLLABORATION_PLUGIN_CONFIG_KEY,

} from '../controller/config.schema';

/**
 * Server protocol format for presence updates
 * This is the format used by the WebSocket server, NOT the internal IUserAwareness format
 */
interface IServerPresenceUpdate {
    clientID: number;
    id: string;
    name: string;
    selectionParams?: {
        unitId: string;
        subUnitId: string;
        selections: Array<{
            range: {
                startRow: number;
                startColumn: number;
                endRow: number;
                endColumn: number;
            };
            primary: unknown;
            style: unknown;
        }>;
    };
}

/**
 * Socket.IO implementation of the network service
 *
 * This runs in the remote context and handles all network communication
 * for the collaboration system.
 */
export class SocketNetworkService
    extends Disposable
    implements INetworkService {
    private _socket: Nullable<Socket>;
    private _config: ICollaborationConfig;

    private readonly _connectionStatus$ =
        new BehaviorSubject<NetworkConnectionStatus>('disconnected');

    private readonly _changesetPushed$ = new Subject<IChangesetPushed>();
    private readonly _awarenessUpdate$ = new Subject<IUserAwareness>();

    readonly connectionStatus$ = this._connectionStatus$.asObservable();
    readonly changesetPushed$ = this._changesetPushed$.asObservable();
    readonly awarenessUpdate$ = this._awarenessUpdate$.asObservable();

    constructor(
        @Inject(IConfigService) private readonly _configService: IConfigService,
        @Inject(ILogService) private readonly _logger: ILogService
    ) {
        super();

        const config =
            this._configService.getConfig<ICollaborationConfig>(
                COLLABORATION_PLUGIN_CONFIG_KEY
            );
        if (!config) {
            throw new Error('Collaboration config not found');
        }
        this._config = config;
    }

    /**
     * Connect to the collaboration server
     */
    async connect(): Promise<void> {
        if (this._socket?.connected) {
            return;
        }

        this._connectionStatus$.next('connecting');

        return new Promise((resolve, reject) => {
            const url = this._config.wsUrl;
            this._socket = io(url, {
                transports: ['websocket'],
                path: '/socket.io/',
                reconnection: true,
                autoConnect: true,
                auth: {
                    token: this._config.accessToken,
                },
            });

            this._socket.on('connect', () => {
                this._logger.log('SocketNetworkService: Connected');
                this._connectionStatus$.next('connected');
                resolve();
            });

            this._socket.on('disconnect', () => {
                this._logger.log('SocketNetworkService: Disconnected');
                this._connectionStatus$.next('disconnected');
            });

            this._socket.on('connect_error', (error) => {
                this._logger.error('SocketNetworkService: Connection error', error);
                this._connectionStatus$.next('disconnected');
                reject(error);
            });

            // Listen for changeset broadcasts from other clients
            this._socket.on('changeset_pushed', (data: IChangesetPushed) => {
                this._logger.log(
                    `SocketNetworkService: Received changeset_pushed for doc ${data.docId}, rev ${data.serverRev}`
                );
                this._changesetPushed$.next(data);
            });

            // Listen for presence updates from other clients
            // Server sends IServerPresenceUpdate format, convert to IUserAwareness
            this._socket.on('presence_update', (data: IServerPresenceUpdate) => {
                const awareness = this._convertServerToUserAwareness(data);
                if (awareness) {
                    this._awarenessUpdate$.next(awareness);
                }
            });
        });
    }

    /**
     * Disconnect from the server
     */
    disconnect(): void {
        if (this._socket) {
            this._socket.disconnect();
            this._socket = null;
            this._connectionStatus$.next('disconnected');
        }
    }

    /**
     * Join a document room
     */
    joinDoc(docId: string): Promise<IJoinDocAck> {
        return new Promise((resolve, reject) => {
            if (!this._socket?.connected) {
                reject(new Error('Socket not connected'));
                return;
            }

            this._socket.emit('join_doc', { docId }, (ack: IJoinDocAck) => {
                if (ack.status === 'ok') {
                    this._logger.log(
                        `SocketNetworkService: Joined doc ${docId}, version: ${ack.version}`
                    );
                    resolve(ack);
                } else {
                    this._logger.error(
                        `SocketNetworkService: Failed to join doc ${docId}: ${ack.message}`
                    );
                    reject(new Error(ack.message || 'Failed to join doc'));
                }
            });
        });
    }

    /**
     * Leave a document room
     */
    leaveDoc(docId: string): void {
        if (this._socket?.connected) {
            this._socket.emit('leave_doc', { docId });
            this._logger.log(`SocketNetworkService: Left doc ${docId}`);
        }
    }

    /**
     * Send a changeset to the server
     */
    sendChangeset(request: IChangesetRequest): Promise<IChangesetAck> {
        return new Promise((resolve, reject) => {
            if (!this._socket?.connected) {
                reject(new Error('Socket not connected'));
                return;
            }

            const requestWithClient: IChangesetRequest = {
                ...request,
                clientId: this._socket.id,
            };

            this._logger.log(
                `SocketNetworkService: Sending changeset for doc ${request.docId}, baseRev ${request.baseRev}, mutations: ${request.mutations.length}`
            );

            this._socket.emit(
                'changeset',
                requestWithClient,
                (ack: IChangesetAck) => {
                    if (ack.status === 'ok') {
                        this._logger.log(
                            `SocketNetworkService: Changeset accepted, serverRev: ${ack.serverRev}`
                        );
                        resolve(ack);
                    } else {
                        this._logger.error(
                            `SocketNetworkService: Changeset rejected: ${ack.message}`
                        );
                        resolve(ack); // Still resolve, let caller handle the error status
                    }
                }
            );
        });
    }

    /**
     * Fetch operations from server
     */
    fetchOps(docId: string, startRev: number): Promise<IFetchOpsResult> {
        return new Promise((resolve, reject) => {
            if (!this._socket?.connected) {
                reject(new Error('Socket not connected'));
                return;
            }

            this._socket.emit(
                'fetch_ops',
                { docId, startRev },
                (ack: IFetchOpsResult) => {
                    if (ack.status === 'ok') {
                        this._logger.log(
                            `SocketNetworkService: Fetched ${ack.operations?.length ?? 0} ops for doc ${docId} since rev ${startRev}`
                        );
                        resolve(ack);
                    } else {
                        this._logger.error(
                            `SocketNetworkService: Failed to fetch ops: ${ack.message}`
                        );
                        reject(new Error(ack.message || 'Failed to fetch ops'));
                    }
                }
            );
        });
    }

    /**
     * Get the client ID
     */
    getClientId(): string | undefined {
        return this._socket?.id;
    }

    /**
     * Check if connected
     */
    isConnected(): boolean {
        return this._socket?.connected ?? false;
    }

    /**
     * Broadcast user awareness state to other clients
     */
    async broadcastAwareness(awareness: IUserAwareness): Promise<void> {
        if (!this._socket?.connected) {
            return;
        }

        // Convert IUserAwareness.selection to ISetSelectionsOperationParams format for server
        const selectionParams = awareness.selection
            ? {
                unitId: awareness.docId,
                subUnitId: awareness.selection.sheetId || '',
                selections:
                        awareness.selection.ranges?.map((range) => ({
                            range: {
                                startRow: range.startRow,
                                startColumn: range.startColumn,
                                endRow: range.endRow,
                                endColumn: range.endColumn,
                            },
                            primary: null,
                            style: null,
                        })) || [],
            }
            : undefined;

        this._socket.volatile.emit('presence_update', {
            docId: awareness.docId,
            clientId: awareness.userId,
            user: { id: awareness.userId, name: awareness.userName },
            selectionParams,
        });
    }

    /**
     * Initialize awareness for a document and get current states
     */
    async initAwareness(docId: string): Promise<IAwarenessInitResult> {
        return new Promise((resolve, reject) => {
            if (!this._socket?.connected) {
                reject(new Error('Socket not connected'));
                return;
            }

            // Server returns states in IServerPresenceUpdate format
            this._socket.emit(
                'awareness_init',
                { docId },
                (ack: { status: string; states: IServerPresenceUpdate[] }) => {
                    if (ack.status === 'ok') {
                        this._logger.log(
                            `SocketNetworkService: Awareness init for doc ${docId}, got ${ack.states?.length ?? 0} states`
                        );
                        // Convert server format to IUserAwareness format
                        const states = (ack.states || [])
                            .map((s) => this._convertServerToUserAwareness(s, docId))
                            .filter((s): s is IUserAwareness => s !== null);
                        resolve({ status: 'ok', states });
                    } else {
                        this._logger.error(
                            `SocketNetworkService: Failed to init awareness for doc ${docId}`
                        );
                        resolve({ status: 'error', states: [] });
                    }
                }
            );
        });
    }

    /**
     * Convert server presence update format to IUserAwareness
     */
    private _convertServerToUserAwareness(
        data: IServerPresenceUpdate,
        docIdOverride?: string
    ): IUserAwareness | null {
        const selectionParams = data.selectionParams;
        const docId = docIdOverride || selectionParams?.unitId;

        if (!docId) {
            return null;
        }

        return {
            docId,
            userId: data.id,
            userName: data.name,
            selection: selectionParams
                ? {
                    sheetId: selectionParams.subUnitId,
                    ranges: selectionParams.selections?.map((sel) => ({
                        startRow: sel.range.startRow,
                        startColumn: sel.range.startColumn,
                        endRow: sel.range.endRow,
                        endColumn: sel.range.endColumn,
                    })),
                }
                : undefined,
            timestamp: Date.now(),
        };
    }

    /**
     * Set connection status (for testing or manual override)
     */
    setConnectionStatus(status: NetworkConnectionStatus): void {
        this._connectionStatus$.next(status);
    }

    override dispose(): void {
        super.dispose();
        this.disconnect();
        this._connectionStatus$.complete();
        this._changesetPushed$.complete();
    }
}
