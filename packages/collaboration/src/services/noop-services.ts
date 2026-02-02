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
    IChangesetPushed as IChangesetPushedType,
    IChangesetRequest,
    IFetchOpsResult,
    IJoinDocAck,
    IUserAwareness,
    NetworkConnectionStatus,
} from '../common/types';
import type { ICollaborationConfig } from '../controller/config.schema';
import type { IAwarenessInitResult, INetworkService } from './network.service';
import type { IChangesetPushed, ISocketService } from './socket.service';
import { Disposable } from '@univerjs/core';
import { BehaviorSubject, Subject } from 'rxjs';

/**
 * Noop SocketService for client-with-remote mode.
 *
 * In remote mode, socket communication happens in the remote context (worker),
 * so the main thread doesn't need an actual socket. This noop implementation
 * satisfies the dependency requirement of CollaborationController.
 */
export class NoopSocketService extends Disposable implements ISocketService {
    private _connected$ = new Subject<void>();
    private _disconnected$ = new Subject<void>();
    private _changesetPushed$ = new Subject<IChangesetPushed>();

    readonly connected$ = this._connected$.asObservable();
    readonly connected = false;
    readonly disconnected$ = this._disconnected$.asObservable();
    readonly changesetPushed$ = this._changesetPushed$.asObservable();

    createSocket(_config: ICollaborationConfig): Nullable<Socket> {
        // In remote mode, socket is created in the worker context
        // Return null to indicate no socket is needed in main thread
        return null;
    }

    getSocket(): Nullable<Socket> {
        return null;
    }

    emit(_event: string, ..._args: unknown[]): void {
        // Noop - socket operations handled in remote context
    }

    override dispose(): void {
        super.dispose();
        this._connected$.complete();
        this._disconnected$.complete();
        this._changesetPushed$.complete();
    }
}

/**
 * Noop NetworkService for client-with-remote mode.
 *
 * In remote mode, network communication happens in the remote context (worker),
 * so the main thread doesn't need an actual network service. This noop implementation
 * satisfies the dependency requirement.
 */
export class NoopNetworkService extends Disposable implements INetworkService {
    private readonly _connectionStatus$ = new BehaviorSubject<NetworkConnectionStatus>('disconnected');
    private readonly _changesetPushed$ = new Subject<IChangesetPushedType>();
    private readonly _awarenessUpdate$ = new Subject<IUserAwareness>();

    readonly connectionStatus$ = this._connectionStatus$.asObservable();
    readonly changesetPushed$ = this._changesetPushed$.asObservable();
    readonly awarenessUpdate$ = this._awarenessUpdate$.asObservable();

    async connect(): Promise<void> {
        // Noop - network operations handled in remote context
    }

    disconnect(): void {
        // Noop
    }

    async joinDoc(_docId: string): Promise<IJoinDocAck> {
        return { status: 'ok', version: 0 };
    }

    leaveDoc(_docId: string): void {
        // Noop
    }

    async sendChangeset(_request: IChangesetRequest): Promise<IChangesetAck> {
        return { status: 'ok', serverRev: 0 };
    }

    async fetchOps(_docId: string, _startRev: number): Promise<IFetchOpsResult> {
        return { status: 'ok', operations: [] };
    }

    getClientId(): string | undefined {
        return undefined;
    }

    isConnected(): boolean {
        return false;
    }

    async broadcastAwareness(_awareness: IUserAwareness): Promise<void> {
        // Noop
    }

    async initAwareness(_docId: string): Promise<IAwarenessInitResult> {
        return { status: 'ok', states: [] };
    }

    setConnectionStatus(status: NetworkConnectionStatus): void {
        this._connectionStatus$.next(status);
    }

    override dispose(): void {
        super.dispose();
        this._connectionStatus$.complete();
        this._changesetPushed$.complete();
        this._awarenessUpdate$.complete();
    }
}
