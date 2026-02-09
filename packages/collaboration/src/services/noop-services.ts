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
    IUserAwareness,
    NetworkConnectionStatus,
} from '../common/types';
import type { IAwarenessInitResult, INetworkService } from './network.service';
import { Disposable } from '@univerjs/core';
import { BehaviorSubject, Subject } from 'rxjs';

/**
 * Noop NetworkService for client-with-remote mode.
 *
 * In remote mode, network communication happens in the remote context (worker),
 * so the main thread doesn't need an actual network service. This noop implementation
 * satisfies the dependency requirement.
 */
export class NoopNetworkService extends Disposable implements INetworkService {
    private readonly _connectionStatus$ = new BehaviorSubject<NetworkConnectionStatus>('disconnected');
    private readonly _changesetPushed$ = new Subject<IChangesetPushed>();
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
        return { status: 'ok', serverRev: 0, opIds: [] };
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
