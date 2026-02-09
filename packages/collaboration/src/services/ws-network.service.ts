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

import type {
    IChangesetAck,
    IChangesetPushed,
    IChangesetRequest,
    IFetchOpsResult,
    IJoinDocAck,
    IOperationInfo,
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
import { COLLABORATION_PLUGIN_CONFIG_KEY } from '../controller/config.schema';

/**
 * Server protocol format for presence updates (from ws-gateway)
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
 * Server message types received from ws-gateway
 */
type ServerMessage =
    | { type: 'join_doc_ack'; requestId: string; status: string; version?: number; message?: string }
    | { type: 'awareness_init_ack'; requestId: string; status: string; states: IServerPresenceUpdate[] }
    | { type: 'changeset_pushed'; docId: string; serverRev: number; userId: string }
    | { type: 'presence_update'; clientID: number; id: string; name: string; selectionParams?: unknown };

/**
 * Flat operation format returned by ot-server HTTP API
 */
interface IFlatOperation {
    rev: number;
    user_id: string;
    mutation_id: string;
    params: unknown;
    op_id: string;
    created_at: string;
}

/**
 * Pending request tracker for WS request-response pattern
 */
interface IPendingRequest<T> {
    resolve: (value: T) => void;
    reject: (reason: Error) => void;
    timer: ReturnType<typeof setTimeout>;
}

const WS_REQUEST_TIMEOUT_MS = 10_000;
const RECONNECT_BASE_DELAY_MS = 1000;
const RECONNECT_MAX_DELAY_MS = 16_000;

/**
 * Native WebSocket + HTTP implementation of INetworkService.
 *
 * Uses:
 * - WebSocket to ws-gateway for: join_doc, leave_doc, presence_update, awareness_init, changeset_pushed
 * - HTTP fetch to ot-server for: sendChangeset (POST), fetchOps (GET)
 */
export class WsNetworkService extends Disposable implements INetworkService {
    private _ws: WebSocket | null = null;
    private _config!: ICollaborationConfig;
    private _clientId: string;

    private readonly _connectionStatus$ = new BehaviorSubject<NetworkConnectionStatus>('disconnected');
    private readonly _changesetPushed$ = new Subject<IChangesetPushed>();
    private readonly _awarenessUpdate$ = new Subject<IUserAwareness>();

    readonly connectionStatus$ = this._connectionStatus$.asObservable();
    readonly changesetPushed$ = this._changesetPushed$.asObservable();
    readonly awarenessUpdate$ = this._awarenessUpdate$.asObservable();

    private _pendingRequests = new Map<string, IPendingRequest<any>>();
    private _reconnectAttempt = 0;
    private _reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    private _shouldReconnect = false;
    private _pingTimer: ReturnType<typeof setTimeout> | null = null;
    private _requestCounter = 0;

    constructor(
        @Inject(IConfigService) private readonly _configService: IConfigService,
        @Inject(ILogService) private readonly _logger: ILogService
    ) {
        super();

        const config = this._configService.getConfig<ICollaborationConfig>(
            COLLABORATION_PLUGIN_CONFIG_KEY
        );
        if (!config) {
            throw new Error('Collaboration config not found');
        }
        this._config = config;
        this._clientId = this._generateClientId();
    }

    async connect(): Promise<void> {
        if (this._ws?.readyState === WebSocket.OPEN) {
            return;
        }

        this._shouldReconnect = true;
        this._connectionStatus$.next('connecting');

        return new Promise((resolve, reject) => {
            const wsUrl = this._buildWsUrl();
            this._logger.log(`WsNetworkService: Connecting to ${wsUrl}`);

            try {
                this._ws = new WebSocket(wsUrl);
            } catch (e) {
                this._connectionStatus$.next('disconnected');
                reject(e);
                return;
            }

            const onOpen = () => {
                this._logger.log('WsNetworkService: Connected');
                this._connectionStatus$.next('connected');
                this._reconnectAttempt = 0;
                cleanup();
                resolve();
            };

            const onError = (event: Event) => {
                this._logger.error('WsNetworkService: Connection error', event);
                cleanup();
                reject(new Error('WebSocket connection failed'));
            };

            const onClose = () => {
                cleanup();
                reject(new Error('WebSocket closed during connection'));
            };

            const cleanup = () => {
                this._ws?.removeEventListener('open', onOpen);
                this._ws?.removeEventListener('error', onError);
                this._ws?.removeEventListener('close', onClose);

                // Re-attach persistent listeners
                if (this._ws) {
                    this._ws.onmessage = (event) => this._handleMessage(event);
                    this._ws.onclose = () => this._handleClose();
                    this._ws.onerror = (event) => {
                        this._logger.error('WsNetworkService: WebSocket error', event);
                    };
                }
            };

            this._ws.addEventListener('open', onOpen);
            this._ws.addEventListener('error', onError);
            this._ws.addEventListener('close', onClose);
        });
    }

    disconnect(): void {
        this._shouldReconnect = false;
        this._clearReconnectTimer();
        this._rejectAllPending('Disconnected');

        if (this._ws) {
            this._ws.onmessage = null;
            this._ws.onclose = null;
            this._ws.onerror = null;
            this._ws.close();
            this._ws = null;
        }

        this._connectionStatus$.next('disconnected');
    }

    async joinDoc(docId: string): Promise<IJoinDocAck> {
        const requestId = this._nextRequestId();
        this._send({ type: 'join_doc', docId, requestId });

        const ack = await this._waitForResponse<{
            type: 'join_doc_ack';
            requestId: string;
            status: string;
            version?: number;
            message?: string;
        }>(requestId);

        if (ack.status === 'ok') {
            this._logger.log(`WsNetworkService: Joined doc ${docId}`);
            return { status: 'ok', version: ack.version };
        }

        this._logger.error(`WsNetworkService: Failed to join doc ${docId}: ${ack.message}`);
        throw new Error(ack.message || 'Failed to join doc');
    }

    leaveDoc(docId: string): void {
        this._send({ type: 'leave_doc', docId });
        this._logger.log(`WsNetworkService: Left doc ${docId}`);
    }

    /**
     * Send changeset via HTTP POST to ot-server
     */
    async sendChangeset(request: IChangesetRequest): Promise<IChangesetAck> {
        const apiBaseUrl = this._config.apiBaseUrl;
        const url = `${apiBaseUrl}/api/documents/${request.docId}/changeset`;

        const mutations = request.mutations.map((m) => ({
            id: m.id,
            params: m.params,
            op_id: crypto.randomUUID(),
        }));

        const body = {
            base_rev: request.baseRev,
            user_id: '', // Server extracts from token
            client_id: request.clientId || this._clientId,
            mutations,
        };

        this._logger.log(
            `WsNetworkService: Sending changeset for doc ${request.docId}, baseRev ${request.baseRev}, mutations: ${mutations.length}`
        );

        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    Authorization: `Bearer ${this._config.accessToken}`,
                },
                body: JSON.stringify(body),
            });

            if (!response.ok) {
                const errorText = await response.text().catch(() => 'Unknown error');
                this._logger.error(`WsNetworkService: Changeset rejected (HTTP ${response.status}): ${errorText}`);
                return {
                    status: 'error',
                    message: `HTTP ${response.status}: ${errorText}`,
                };
            }

            const result = await response.json() as {
                success: boolean;
                server_rev: number;
                op_ids: string[];
                error?: string;
            };

            if (result.success) {
                this._logger.log(`WsNetworkService: Changeset accepted, serverRev: ${result.server_rev}`);
                return {
                    status: 'ok',
                    serverRev: result.server_rev,
                    opIds: result.op_ids,
                };
            }

            return {
                status: 'error',
                message: result.error || 'Changeset rejected',
            };
        } catch (error) {
            this._logger.error('WsNetworkService: Failed to send changeset', error);
            return {
                status: 'error',
                message: error instanceof Error ? error.message : 'Network error',
            };
        }
    }

    /**
     * Fetch operations via HTTP GET from ot-server.
     *
     * The server returns flat format: [{rev, user_id, mutation_id, params, op_id}]
     * We group by rev into IOperationInfo[]: [{rev, userId, mutations: [{id, params}]}]
     */
    async fetchOps(docId: string, startRev: number): Promise<IFetchOpsResult> {
        const apiBaseUrl = this._config.apiBaseUrl;
        const url = `${apiBaseUrl}/api/documents/${docId}/operations?from_rev=${startRev}`;

        this._logger.log(`WsNetworkService: Fetching ops for doc ${docId}, startRev=${startRev}`);

        try {
            const response = await fetch(url, {
                headers: {
                    Authorization: `Bearer ${this._config.accessToken}`,
                },
            });

            if (!response.ok) {
                const errorText = await response.text().catch(() => 'Unknown error');
                this._logger.error(`WsNetworkService: Fetch ops failed (HTTP ${response.status}): ${errorText}`);
                return { status: 'error', message: `HTTP ${response.status}: ${errorText}` };
            }

            const result = await response.json() as { operations: IFlatOperation[] };
            const operations = this._groupOperationsByRev(result.operations || []);

            this._logger.log(`WsNetworkService: Fetched ${operations.length} grouped ops for doc ${docId}`);

            return { status: 'ok', operations };
        } catch (error) {
            this._logger.error('WsNetworkService: Failed to fetch ops', error);
            return {
                status: 'error',
                message: error instanceof Error ? error.message : 'Network error',
            };
        }
    }

    getClientId(): string | undefined {
        return this._clientId;
    }

    isConnected(): boolean {
        return this._ws?.readyState === WebSocket.OPEN;
    }

    async broadcastAwareness(awareness: IUserAwareness): Promise<void> {
        if (!this.isConnected()) return;

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

        this._send({
            type: 'presence_update',
            docId: awareness.docId,
            user: { id: awareness.userId, name: awareness.userName || '' },
            selectionParams,
        });
    }

    async initAwareness(docId: string): Promise<IAwarenessInitResult> {
        const requestId = this._nextRequestId();
        this._send({ type: 'awareness_init', docId, requestId });

        const ack = await this._waitForResponse<{
            type: 'awareness_init_ack';
            requestId: string;
            status: string;
            states: IServerPresenceUpdate[];
        }>(requestId);

        if (ack.status === 'ok') {
            const states = (ack.states || [])
                .map((s) => this._convertServerToUserAwareness(s, docId))
                .filter((s): s is IUserAwareness => s !== null);
            return { status: 'ok', states };
        }

        return { status: 'error', states: [] };
    }

    setConnectionStatus(status: NetworkConnectionStatus): void {
        this._connectionStatus$.next(status);
    }

    override dispose(): void {
        super.dispose();
        this.disconnect();
        this._connectionStatus$.complete();
        this._changesetPushed$.complete();
        this._awarenessUpdate$.complete();
    }

    // ─── Private Helpers ─────────────────────────────────────

    private _buildWsUrl(): string {
        const base = this._config.wsUrl;
        const separator = base.includes('?') ? '&' : '?';
        return `${base}${separator}token=${encodeURIComponent(this._config.accessToken)}`;
    }

    private _send(data: unknown): void {
        if (this._ws?.readyState === WebSocket.OPEN) {
            this._ws.send(JSON.stringify(data));
        }
    }

    private _nextRequestId(): string {
        return `req_${++this._requestCounter}_${Date.now()}`;
    }

    private _waitForResponse<T extends { requestId: string }>(requestId: string): Promise<T> {
        return new Promise((resolve, reject) => {
            const timer = setTimeout(() => {
                this._pendingRequests.delete(requestId);
                reject(new Error(`Request ${requestId} timed out`));
            }, WS_REQUEST_TIMEOUT_MS);

            this._pendingRequests.set(requestId, { resolve, reject, timer });
        });
    }

    private _handleMessage(event: MessageEvent): void {
        let msg: ServerMessage;
        try {
            msg = JSON.parse(event.data as string);
        } catch {
            this._logger.error('WsNetworkService: Failed to parse message', event.data);
            return;
        }

        switch (msg.type) {
            case 'join_doc_ack':
            case 'awareness_init_ack': {
                const pending = this._pendingRequests.get(msg.requestId);
                if (pending) {
                    clearTimeout(pending.timer);
                    this._pendingRequests.delete(msg.requestId);
                    pending.resolve(msg);
                }
                break;
            }

            case 'changeset_pushed':
                this._changesetPushed$.next({
                    docId: msg.docId,
                    serverRev: msg.serverRev,
                    userId: msg.userId,
                });
                break;

            case 'presence_update': {
                const awareness = this._convertServerToUserAwareness(
                    msg as unknown as IServerPresenceUpdate
                );
                if (awareness) {
                    this._awarenessUpdate$.next(awareness);
                }
                break;
            }
        }
    }

    private _handleClose(): void {
        this._logger.log('WsNetworkService: Connection closed');
        this._connectionStatus$.next('disconnected');
        this._rejectAllPending('Connection closed');

        if (this._shouldReconnect) {
            this._scheduleReconnect();
        }
    }

    private _scheduleReconnect(): void {
        this._clearReconnectTimer();

        const delay = Math.min(
            RECONNECT_BASE_DELAY_MS * 2 ** this._reconnectAttempt,
            RECONNECT_MAX_DELAY_MS
        );
        this._reconnectAttempt++;

        this._logger.log(`WsNetworkService: Reconnecting in ${delay}ms (attempt ${this._reconnectAttempt})`);

        this._reconnectTimer = setTimeout(async () => {
            try {
                await this.connect();
            } catch {
                // connect() failure will trigger _handleClose which schedules next reconnect
            }
        }, delay);
    }

    private _clearReconnectTimer(): void {
        if (this._reconnectTimer) {
            clearTimeout(this._reconnectTimer);
            this._reconnectTimer = null;
        }
    }

    private _rejectAllPending(reason: string): void {
        this._pendingRequests.forEach((req) => {
            clearTimeout(req.timer);
            req.reject(new Error(reason));
        });
        this._pendingRequests.clear();
    }

    /**
     * Group flat operations by rev into IOperationInfo format.
     *
     * Server returns: [{rev, user_id, mutation_id, params, op_id}]
     * Client expects: [{rev, userId, mutations: [{id, params}]}]
     */
    private _groupOperationsByRev(flatOps: IFlatOperation[]): IOperationInfo[] {
        const grouped = new Map<number, { userId: string; mutations: Array<{ id: string; params: unknown }> }>();

        for (const op of flatOps) {
            let group = grouped.get(op.rev);
            if (!group) {
                group = { userId: op.user_id, mutations: [] };
                grouped.set(op.rev, group);
            }
            group.mutations.push({ id: op.mutation_id, params: op.params });
        }

        return Array.from(grouped.entries())
            .sort(([a], [b]) => a - b)
            .map(([rev, group]) => ({
                rev,
                userId: group.userId,
                mutations: group.mutations,
            }));
    }

    private _convertServerToUserAwareness(
        data: IServerPresenceUpdate,
        docIdOverride?: string
    ): IUserAwareness | null {
        const selectionParams = data.selectionParams;
        const docId = docIdOverride || selectionParams?.unitId;

        if (!docId) return null;

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

    private _generateClientId(): string {
        return crypto.randomUUID();
    }
}
