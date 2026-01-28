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

import type { IMutationInfo, Nullable } from '@univerjs/core';
import type { Observable } from 'rxjs';
import type { Socket } from 'socket.io-client';
import { createIdentifier, Disposable, ILogService } from '@univerjs/core';
import { Subject } from 'rxjs';
import { io } from 'socket.io-client';

export interface ISocketService {
    connected$: Observable<void>;
    connected: boolean;
    disconnected$: Observable<void>;
    changesetPushed$: Observable<IChangesetPushed>;
    createSocket(url: string): Nullable<Socket>;
    getSocket(): Nullable<Socket>;
    emit(event: string, ...args: any[]): void;
}

export interface IChangeset {
    unitId: string;
    baseRev: number;
    mutations: IMutationInfo[];
}

// Socket.IO event types matching server implementation
export interface IJoinDocRequest {
    docId: string;
}

export interface IJoinDocAck {
    status: string;
    version?: number;
    content?: Record<string, unknown>;
    message?: string;
}

export interface IChangesetRequest {
    baseRev: number;
    mutations: IMutationInfo[];
    docId: string;
    clientId?: string;
}

export interface IChangesetAck {
    status: string;
    serverRev?: number;
    /**
     * The mutations after server-side OT transformation
     * This is the "authoritative" result from server that clients should use to ensure consistency
     */
    mutations?: IMutationInfo[];
    message?: string;
}

export interface IChangesetPushed {
    docId: string;
    serverRev: number;
    userId: string;
    mutations: IMutationInfo[];
}

export interface IFetchOpsRequest {
    docId: string;
    startRev: number;
}

export interface IOperationInfo {
    rev: number;
    userId: string;
    mutations: IMutationInfo[];
}

export interface IFetchOpsAck {
    status: string;
    operations?: IOperationInfo[];
    message?: string;
}

export const ISocketService = createIdentifier<ISocketService>('univer.collaboration.socket.service');

export class SocketService extends Disposable implements ISocketService {
    private _socket?: Nullable<Socket>;
    private _connected$: Subject<void> = new Subject();
    connected$ = this._connected$.asObservable();
    connected = false;
    private _disconnected$: Subject<void> = new Subject();
    disconnected$ = this._disconnected$.asObservable();
    private _changesetPushed$: Subject<IChangesetPushed> = new Subject();
    changesetPushed$ = this._changesetPushed$.asObservable();

    constructor(
        @ILogService private readonly _logger: ILogService
    ) {
        super();
    }

    createSocket(url: string): Nullable<Socket> {
        this._socket = io(url, {
            transports: ['websocket'],
            // Explicitly specify root namespace
            path: '/socket.io/',
            reconnection: true,
            autoConnect: true,
        });

        this._socket.on('connect', () => {
            this._logger.log('Socket.IO connected');
            this.connected = true;
            this._connected$.next();
        });

        this._socket.on('disconnect', () => {
            this._logger.log('Socket.IO disconnected');
            this.connected = false;
            this._disconnected$.next();
        });

        this._socket.on('connect_error', (error) => {
            this._logger.error('Socket.IO connection error:', error);
        });

        // Listen for changeset_pushed events (from other clients)
        this._socket.on('changeset_pushed', (data: IChangesetPushed) => {
            this._logger.log(`Received changeset_pushed for doc ${data.docId}, rev ${data.serverRev}`);
            this._changesetPushed$.next(data);
        });

        return this._socket;
    }

    getSocket(): Nullable<Socket> {
        return this._socket;
    }

    emit(event: string, ...args: any[]): void {
        this._socket?.emit(event, ...args);
    }

    override dispose(): void {
        super.dispose();
        this._connected$.complete();
        this._disconnected$.complete();
        this._changesetPushed$.complete();
        this._socket?.disconnect();
    }
}
