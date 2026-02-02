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

import type { IUserAwareness } from '../common/types';
import {
    createIdentifier,
    Disposable,
    ILogService,
    Inject,
} from '@univerjs/core';
import { fromModule, IRPCChannelService, toModule } from '@univerjs/rpc';
import { BehaviorSubject, Subject } from 'rxjs';
import { AWARENESS_CALLBACK_SERVICE_NAME, AWARENESS_REMOTE_SERVICE_NAME } from '../common/types';
import { INetworkService } from './network.service';

/**
 * Remote awareness service interface (runs in worker thread)
 *
 * This service manages user awareness state (presence, selection, cursor)
 * and communicates with the network service to broadcast updates.
 *
 * RPC Serialization Notes:
 * - Arrays/Objects: Supported
 * - Map/Set: NOT supported, use arrays instead
 * - Observable: NOT used over RPC - use callback service instead
 */
export interface IAwarenessRemoteService {
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
}

export const IAwarenessRemoteService =
    createIdentifier<IAwarenessRemoteService>('univer.awareness-remote.service');

// ============================================================================
// Callback Interface (Main Thread -> Worker calls back)
// ============================================================================

/**
 * Awareness callback service interface (runs in main thread, called by worker)
 *
 * This service receives awareness updates from the worker thread.
 * The worker calls this when it receives awareness updates from the network.
 */
export interface IAwarenessCallbackService {
    /**
     * Called by worker when remote awareness update is received from network
     * @param awareness User awareness state
     */
    onRemoteAwarenessUpdate(awareness: IUserAwareness): void;
}

export const IAwarenessCallbackService =
    createIdentifier<IAwarenessCallbackService>('univer.awareness-callback.service');

// ============================================================================
// Main Thread Proxy Service
// ============================================================================

/**
 * Main thread awareness remote service proxy
 *
 * This is a thin proxy to the AwarenessRemoteService running in worker.
 * It forwards all awareness operations via RPC.
 * It also implements IAwarenessCallbackService to receive callbacks from worker.
 */
export class AwarenessRemoteProxyService
    extends Disposable
    implements IAwarenessRemoteService, IAwarenessCallbackService {
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
        this._registerCallbackService();
        this._initRemoteService();
    }

    /**
     * Register this service as a callback for worker to call
     */
    private _registerCallbackService(): void {
        this._rpcChannelService.registerChannel(
            AWARENESS_CALLBACK_SERVICE_NAME,
            fromModule(this as IAwarenessCallbackService)
        );
        this._logger.log(
            'AwarenessRemoteProxyService: Registered callback service'
        );
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

    // ========================================================================
    // IAwarenessCallbackService implementation (called by worker)
    // ========================================================================

    onRemoteAwarenessUpdate(awareness: IUserAwareness): void {
        this._awarenessUpdate$.next(awareness);
        this._logger.log(
            `AwarenessRemoteProxyService: Received awareness callback for ${awareness.docId} from user ${awareness.userId}`
        );
    }

    // ========================================================================
    // IAwarenessRemoteService implementation (calls to worker)
    // ========================================================================

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
 * - Receives and forwards remote awareness updates via callback to main thread
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

    // Callback service to notify main thread
    private _callbackService: IAwarenessCallbackService | null = null;

    constructor(
        @Inject(INetworkService) private readonly _networkService: INetworkService,
        @Inject(IRPCChannelService)
        private readonly _rpcChannelService: IRPCChannelService,
        @Inject(ILogService) private readonly _logger: ILogService
    ) {
        super();
        this._initCallbackService();
        this._initNetworkListeners();
    }

    /**
     * Get reference to main thread callback service
     */
    private _initCallbackService(): void {
        try {
            const channel = this._rpcChannelService.requestChannel(
                AWARENESS_CALLBACK_SERVICE_NAME
            );
            this._callbackService = toModule<IAwarenessCallbackService>(channel);
            this._logger.log(
                'AwarenessRemoteService: Connected to callback service'
            );
        } catch (error) {
            this._logger.error(
                'AwarenessRemoteService: Failed to connect to callback service',
                error
            );
        }
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

        // Notify main thread via callback (direct method call, not Observable)
        if (this._callbackService) {
            this._callbackService.onRemoteAwarenessUpdate(awareness);
        }

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

    override dispose(): void {
        super.dispose();
        this._documentAwareness.clear();
    }
}
