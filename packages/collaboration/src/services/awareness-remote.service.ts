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

import type { Observable } from 'rxjs';
import type { IUserAwareness } from '../common/types';
import {
    createIdentifier,
    Disposable,
    ILogService,
    Inject,
} from '@univerjs/core';
import { IRPCChannelService, toModule } from '@univerjs/rpc';
import { BehaviorSubject, Subject } from 'rxjs';
import { AWARENESS_REMOTE_SERVICE_NAME } from '../common/types';
import { INetworkService } from './network.service';

/**
 * Remote awareness service interface (runs in worker thread)
 *
 * This service manages user awareness state (presence, selection, cursor)
 * and communicates with the network service to broadcast updates.
 *
 * RPC Serialization Notes:
 * - Observable: Supported by Univer RPC
 * - Arrays/Objects: Supported
 * - Map/Set: NOT supported, use arrays instead
 * - Synchronous methods preferred over Promise when possible
 */
export interface IAwarenessRemoteService {
    /**
     * Observable for awareness updates from other users
     * Emits when remote users' awareness state changes
     */
    awarenessUpdate$: Observable<IUserAwareness>;

    /**
     * Initialize awareness for a document
     * @param docId Document identifier
     * @returns Promise with current awareness states
     */
    initAwareness(docId: string): Promise<IUserAwareness[]>;

    /**
     * Set local user's awareness state and broadcast to network
     * @param docId Document identifier
     * @param awareness User awareness state
     */
    setLocalAwareness(docId: string, awareness: IUserAwareness): void;

    /**
     * Set remote user's awareness state (received from network)
     * @param docId Document identifier
     * @param awareness User awareness state
     */
    setRemoteAwareness(docId: string, awareness: IUserAwareness): void;
}

export const IAwarenessRemoteService =
    createIdentifier<IAwarenessRemoteService>('univer.awareness-remote.service');

// ============================================================================
// Main Thread Proxy Service
// ============================================================================

/**
 * Main thread awareness remote service proxy
 *
 * This is a thin proxy to the AwarenessRemoteService running in worker.
 * It forwards all awareness operations via RPC.
 */
export class AwarenessRemoteProxyService
    extends Disposable
    implements IAwarenessRemoteService {
    private _remoteService: IAwarenessRemoteService | null = null;
    private _init$ = new BehaviorSubject<boolean>(false);

    private readonly _awarenessUpdate$ = new Subject<IUserAwareness>();
    readonly awarenessUpdate$ = this._awarenessUpdate$.asObservable();

    constructor(
        @Inject(IRPCChannelService)
        private readonly _rpcChannelService: IRPCChannelService,
        @Inject(ILogService) private readonly _logger: ILogService
    ) {
        super();
        this._initRemoteService();
    }

    private _initRemoteService(): void {
        this._initRemoteServiceAsync();
    }

    private async _initRemoteServiceAsync(): Promise<void> {
        try {
            const channel = this._rpcChannelService.requestChannel(
                AWARENESS_REMOTE_SERVICE_NAME
            );
            this._remoteService = toModule<IAwarenessRemoteService>(channel);

            // Subscribe to awareness updates from remote
            try {
                const awarenessUpdate$ = this._remoteService.awarenessUpdate$;
                if (
                    awarenessUpdate$ &&
                    typeof awarenessUpdate$.subscribe === 'function'
                ) {
                    this.disposeWithMe(
                        awarenessUpdate$.subscribe((awareness) => {
                            this._awarenessUpdate$.next(awareness);
                        })
                    );
                }
            } catch (subscriptionError) {
                this._logger.warn(
                    'AwarenessRemoteProxyService: Could not subscribe to awarenessUpdate$',
                    subscriptionError
                );
            }

            this._logger.log(
                'AwarenessRemoteProxyService: Connected to remote service'
            );
            this._init$.next(true);
        } catch (error) {
            this._logger.error(
                'AwarenessRemoteProxyService: Failed to connect to remote service',
                error
            );
        }
    }

    async initAwareness(docId: string): Promise<IUserAwareness[]> {
        if (!this._init$.value || !this._remoteService) {
            return [];
        }
        return await this._remoteService.initAwareness(docId);
    }

    setLocalAwareness(docId: string, awareness: IUserAwareness): void {
        if (!this._init$.value || !this._remoteService) {
            return;
        }
        this._remoteService.setLocalAwareness(docId, awareness);
    }

    setRemoteAwareness(docId: string, awareness: IUserAwareness): void {
        if (!this._init$.value || !this._remoteService) {
            return;
        }
        this._remoteService.setRemoteAwareness(docId, awareness);
    }

    override dispose(): void {
        super.dispose();
        this._awarenessUpdate$.complete();
        this._init$.complete();
    }
}

// ============================================================================
// Worker Side Service Implementation
// ============================================================================

/**
 * Awareness remote service implementation (runs in worker thread)
 *
 * Responsibilities:
 * - Manages awareness state per document
 * - Broadcasts local awareness to network
 * - Receives and forwards remote awareness updates
 * - Exposes RPC interface for main thread
 */
export class AwarenessRemoteService
    extends Disposable
    implements IAwarenessRemoteService {
    // Store awareness states per document
    // docId -> userId -> awareness
    private readonly _documentAwareness = new Map<
        string,
        Map<string, IUserAwareness>
    >();

    private readonly _awarenessUpdate$ = new Subject<IUserAwareness>();
    readonly awarenessUpdate$ = this._awarenessUpdate$.asObservable();

    constructor(
        @Inject(INetworkService) private readonly _networkService: INetworkService,
        @Inject(ILogService) private readonly _logger: ILogService
    ) {
        super();
        this._initNetworkListeners();
    }

    private _initNetworkListeners(): void {
        // Listen to awareness updates from network
        this.disposeWithMe(
            this._networkService.awarenessUpdate$.subscribe((awareness) => {
                this._handleRemoteAwareness(awareness);
            })
        );
    }

    private _handleRemoteAwareness(awareness: IUserAwareness): void {
        const { docId, userId } = awareness;
        if (!docId) return;

        // Store in local map
        let docMap = this._documentAwareness.get(docId);
        if (!docMap) {
            docMap = new Map();
            this._documentAwareness.set(docId, docMap);
        }
        docMap.set(userId, awareness);

        // Forward to main thread subscribers
        this._awarenessUpdate$.next(awareness);

        this._logger.log(
            `AwarenessRemoteService: Received awareness update for ${docId} from user ${userId}`
        );
    }

    async initAwareness(docId: string): Promise<IUserAwareness[]> {
        this._logger.log(`AwarenessRemoteService: Init awareness for ${docId}`);

        // Check if socket is connected
        if (!this._networkService.isConnected()) {
            this._logger.log(
                `AwarenessRemoteService: Socket not connected, returning empty array for ${docId}`
            );
            return [];
        }

        // Try to fetch from network
        try {
            const result = await this._networkService.initAwareness(docId);
            if (result.status === 'ok') {
                // Store received states
                const docMap = new Map<string, IUserAwareness>();
                for (const awareness of result.states) {
                    docMap.set(awareness.userId, awareness);
                }
                this._documentAwareness.set(docId, docMap);

                this._logger.log(
                    `AwarenessRemoteService: Loaded ${result.states.length} awareness states for ${docId}`
                );

                return result.states;
            }
            return [];
        } catch (error) {
            this._logger.error(
                `AwarenessRemoteService: Failed to init awareness for ${docId}`,
                error
            );
            return [];
        }
    }

    setLocalAwareness(docId: string, awareness: IUserAwareness): void {
        this._logger.log(
            `AwarenessRemoteService: Set local awareness for ${docId}`
        );

        // Store locally
        let docMap = this._documentAwareness.get(docId);
        if (!docMap) {
            docMap = new Map();
            this._documentAwareness.set(docId, docMap);
        }
        docMap.set(awareness.userId, awareness);

        // Broadcast to network (async, don't block RPC call)
        this._networkService.broadcastAwareness(awareness).catch((error) => {
            this._logger.error(
                'AwarenessRemoteService: Failed to broadcast awareness',
                error
            );
        });
    }

    setRemoteAwareness(docId: string, awareness: IUserAwareness): void {
        this._handleRemoteAwareness(awareness);
    }

    override dispose(): void {
        super.dispose();
        this._awarenessUpdate$.complete();
        this._documentAwareness.clear();
    }
}
