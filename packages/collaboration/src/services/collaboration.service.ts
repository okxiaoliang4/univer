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

import type { IMutationInfo, UnitModel } from '@univerjs/core';
import type { Observable } from 'rxjs';
import type {
    IDocumentSyncState,
    NetworkConnectionStatus,
} from '../common/types';
import type {
    IDocumentMutationApplier,
    IDocumentNetworkOperations,
    IDocumentPersistence,
    IDocumentTransformOperations,
} from '../models/collaboration-document.model';
import {
    CommandType,
    createIdentifier,
    Disposable,
    ICommandService,
    ILogService,
    isInternalEditorID,
    IUniverInstanceService,
    UniverInstanceType,
} from '@univerjs/core';
import { fromModule, IRPCChannelService, toModule } from '@univerjs/rpc';
import { BehaviorSubject } from 'rxjs';
import { COLLABORATION_CALLBACK_SERVICE_NAME, COLLABORATION_SERVICE_NAME } from '../common/types';
import { CollaborationDocumentModel } from '../models/collaboration-document.model';
import { INetworkService } from './network.service';
import { IPendingMutationSerivce } from './offline-storage.service';
import { ITransformService } from './transform.service';

/**
 * Unified collaboration service interface (RPC interface for main → worker calls)
 *
 * This interface is isomorphic - it can be implemented by:
 * 1. A local service running in the same thread (main-only mode)
 * 2. A remote service exposed via RPC (worker mode)
 * 3. A proxy service that delegates to a remote via RPC (main thread client)
 *
 * RPC Serialization Notes:
 * - Observable: NOT used over RPC - use callback service instead
 * - For real-time state updates, worker calls ICollaborationCallbackService
 */
export interface ICollaborationService {
    /**
     * Observable for connection status changes
     */
    connectionStatus$: Observable<NetworkConnectionStatus>;

    /**
     * Get current document sync state
     */
    getDocumentState(docId: string): IDocumentSyncState;

    /**
     * Get observable for document sync state changes
     *
     * Note: This Observable is NOT transferred over RPC. In worker mode,
     * the main thread proxy maintains its own local BehaviorSubject that
     * receives updates via ICollaborationCallbackService.onDocumentStateChange().
     */
    getDocumentState$(docId: string): Observable<IDocumentSyncState>;

    /**
     * Get current saved status for a document (true = no pending)
     */
    getSavedStatus(docId: string): boolean;

    /**
     * Check if a document is currently synced
     */
    isDocumentSynced(docId: string): boolean;

    /**
     * Get the current server revision for a document
     */
    getServerRev(docId: string): number;

    /**
     * Manually flush pending mutations for a document
     */
    flush(docId: string): void;

    /**
     * Reset a document to clean state (discards pending)
     */
    reset(docId: string): void;

    /**
     * Send a changeset to the server
     * @deprecated In remote-first architecture, mutations are automatically
     * captured and sent by the remote context.
     */
    sendChangeset?(changeset: { unitId: string; baseRev: number; mutations: IMutationInfo[] }): Promise<void>;
}

export const ICollaborationService = createIdentifier<ICollaborationService>(
    'univer.collaboration.service'
);

// ============================================================================
// Callback Interface (Worker calls Main Thread)
// ============================================================================

/**
 * Collaboration callback service interface (runs in main thread, called by worker)
 *
 * This service receives state updates from the worker thread.
 * The worker calls this when connection status or document state changes.
 */
export interface ICollaborationCallbackService {
    /**
     * Called by worker when connection status changes
     */
    onConnectionStatusChange(status: NetworkConnectionStatus): void;

    /**
     * Called by worker when document sync state changes
     */
    onDocumentStateChange(docId: string, state: IDocumentSyncState): void;

    /**
     * Called by worker when document saved status changes
     */
    onSavedStatusChange(docId: string, saved: boolean): void;
}

export const ICollaborationCallbackService =
    createIdentifier<ICollaborationCallbackService>('univer.collaboration-callback.service');

// ============================================================================
// Main Thread Proxy Service (for useRemote: true mode)
// ============================================================================

/**
 * Main thread collaboration service proxy
 *
 * This is a thin proxy to the CollaborationService running in a remote context.
 * All collaboration logic (network, OT, state machine) runs in the remote context.
 *
 * Benefits of this architecture:
 * - Heavy WASM transforms don't block the main thread
 * - Network latency handling doesn't affect UI responsiveness
 * - Remote mutations are applied via ICommandService in remote context and
 *   auto-sync to main thread via Univer's DataSyncReplicaController
 *
 * It also implements ICollaborationCallbackService to receive state updates from worker.
 */
export class CollaborationProxyService
    extends Disposable
    implements ICollaborationService, ICollaborationCallbackService {
    private _remoteService: ICollaborationService | null = null;
    private _init$ = new BehaviorSubject<boolean>(false);

    // Local state maintained via callbacks from worker
    private readonly _connectionStatus$ =
        new BehaviorSubject<NetworkConnectionStatus>('disconnected');

    private readonly _documentStateSubjects = new Map<
        string,
        BehaviorSubject<IDocumentSyncState>
    >();

    private readonly _savedStatusSubjects = new Map<
        string,
        BehaviorSubject<boolean>
    >();

    // Public observables for main thread consumers (NOT over RPC)
    readonly connectionStatus$ = this._connectionStatus$.asObservable();

    constructor(
        @IRPCChannelService private readonly _rpcChannelService: IRPCChannelService,
        @ILogService private readonly _logger: ILogService
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
            COLLABORATION_CALLBACK_SERVICE_NAME,
            fromModule(this as ICollaborationCallbackService)
        );
        this._logger.log(
            'CollaborationProxyService: Registered callback service'
        );
    }

    private _initRemoteService(): void {
        this._initRemoteServiceAsync();
    }

    private async _initRemoteServiceAsync(): Promise<void> {
        try {
            const channel = this._rpcChannelService.requestChannel(
                COLLABORATION_SERVICE_NAME
            );
            this._remoteService = toModule<ICollaborationService>(channel);

            this._logger.log(
                'CollaborationProxyService: Connected to remote service'
            );
            this._init$.next(true);
        } catch (error) {
            this._logger.error(
                'CollaborationProxyService: Failed to connect to remote service',
                error
            );
        }
    }

    // ========================================================================
    // ICollaborationCallbackService implementation (called by worker)
    // ========================================================================

    onConnectionStatusChange(status: NetworkConnectionStatus): void {
        this._connectionStatus$.next(status);
        this._logger.log(
            `CollaborationProxyService: Connection status changed to ${status}`
        );
    }

    onDocumentStateChange(docId: string, state: IDocumentSyncState): void {
        let subject = this._documentStateSubjects.get(docId);
        if (!subject) {
            subject = new BehaviorSubject<IDocumentSyncState>(state);
            this._documentStateSubjects.set(docId, subject);
        } else {
            subject.next(state);
        }
    }

    onSavedStatusChange(docId: string, saved: boolean): void {
        let subject = this._savedStatusSubjects.get(docId);
        if (!subject) {
            subject = new BehaviorSubject<boolean>(saved);
            this._savedStatusSubjects.set(docId, subject);
        } else {
            subject.next(saved);
        }
    }

    // ========================================================================
    // ICollaborationService implementation (calls to worker)
    // ========================================================================

    getConnectionStatus(): NetworkConnectionStatus {
        return this._connectionStatus$.value;
    }

    getDocumentState(docId: string): IDocumentSyncState {
        const subject = this._documentStateSubjects.get(docId);
        return subject?.value ?? {
            state: 'synced',
            serverRev: 0,
            pendingCount: 0,
            awaitingCount: 0,
        };
    }

    getSavedStatus(docId: string): boolean {
        const subject = this._savedStatusSubjects.get(docId);
        return subject?.value ?? true;
    }

    // Observable getters for main thread consumers (NOT part of RPC interface)
    getDocumentState$(docId: string): Observable<IDocumentSyncState> {
        let subject = this._documentStateSubjects.get(docId);
        if (!subject) {
            subject = new BehaviorSubject<IDocumentSyncState>({
                state: 'synced',
                serverRev: 0,
                pendingCount: 0,
                awaitingCount: 0,
            });
            this._documentStateSubjects.set(docId, subject);
        }
        return subject.asObservable();
    }

    getSavedStatus$(docId: string): Observable<boolean> {
        let subject = this._savedStatusSubjects.get(docId);
        if (!subject) {
            subject = new BehaviorSubject<boolean>(true);
            this._savedStatusSubjects.set(docId, subject);
        }
        return subject.asObservable();
    }

    isDocumentSynced(docId: string): boolean {
        return this._remoteService?.isDocumentSynced(docId) ?? true;
    }

    getServerRev(docId: string): number {
        return this._remoteService?.getServerRev(docId) ?? 0;
    }

    flush(docId: string): void {
        this._remoteService?.flush(docId);
    }

    reset(docId: string): void {
        this._remoteService?.reset(docId);
    }

    override dispose(): void {
        super.dispose();
        this._connectionStatus$.complete();
        this._documentStateSubjects.forEach((subject) => subject.complete());
        this._documentStateSubjects.clear();
        this._savedStatusSubjects.forEach((subject) => subject.complete());
        this._savedStatusSubjects.clear();
    }
}

// ============================================================================
// Worker/Server Side Service (for isRemoteSide: true mode)
// ============================================================================

/**
 * Collaboration service implementation (runs in worker or main thread)
 *
 * Responsibilities:
 * - Manages CollaborationDocumentModel instances per document
 * - Listens to local mutations via ICommandService.onMutationExecutedForCollab
 * - Coordinates with network service for sending/receiving changesets
 * - Executes remote mutations locally (auto-syncs to main via Univer RPC)
 * - Notifies main thread of state changes via callback service
 */
export class CollaborationService
    extends Disposable
    implements ICollaborationService {
    private readonly _documentModels = new Map<
        string,
        CollaborationDocumentModel
    >();

    private readonly _joinedDocs = new Set<string>();

    private readonly _connectionStatus$ =
        new BehaviorSubject<NetworkConnectionStatus>('disconnected');

    readonly connectionStatus$ = this._connectionStatus$.asObservable();

    private readonly _documentStateSubjects = new Map<
        string,
        BehaviorSubject<IDocumentSyncState>
    >();

    private readonly _lastDocumentStates = new Map<
        string,
        IDocumentSyncState['state']
    >();

    private readonly _savedStatusSubjects = new Map<
        string,
        BehaviorSubject<boolean>
    >();

    // Debounce timers for flushing
    private readonly _flushTimers = new Map<
        string,
        ReturnType<typeof setTimeout>
    >();

    private readonly _flushDelay = 100; // ms

    // Callback service to notify main thread
    private _callbackService: ICollaborationCallbackService | null = null;

    constructor(
        @ICommandService private readonly _commandService: ICommandService,
        @INetworkService private readonly _networkService: INetworkService,
        @ITransformService private readonly _transformService: ITransformService,
        @IUniverInstanceService private readonly _univerInstanceService: IUniverInstanceService,
        @IRPCChannelService private readonly _rpcChannelService: IRPCChannelService,
        @ILogService private readonly _logger: ILogService,
        @IPendingMutationSerivce private readonly _pendingMutationService: IPendingMutationSerivce
    ) {
        super();
        this._initCallbackService();
        this._init();
    }

    /**
     * Get reference to main thread callback service
     */
    private _initCallbackService(): void {
        try {
            const channel = this._rpcChannelService.requestChannel(
                COLLABORATION_CALLBACK_SERVICE_NAME
            );
            this._callbackService = toModule<ICollaborationCallbackService>(channel);
            this._logger.log(
                'CollaborationService: Connected to callback service'
            );
        } catch (error) {
            this._logger.error(
                'CollaborationService: Failed to connect to callback service',
                error
            );
        }
    }

    private _init(): void {
        this._initNetworkListeners();
        this._initMutationListeners();
        this._initInstanceListeners();
    }

    private _initNetworkListeners(): void {
        this.disposeWithMe(
            this._networkService.connectionStatus$.subscribe((status) => {
                this._connectionStatus$.next(status);

                // Notify main thread via callback
                if (this._callbackService) {
                    this._callbackService.onConnectionStatusChange(status);
                }

                if (status === 'connected') {
                    this._documentModels.forEach((model) => model.onNetworkConnected());
                    this._rejoinDocuments();
                    // Schedule flush for all documents with pending mutations
                    // This handles offline edits that need to be synced after reconnection
                    this._flushPendingAfterReconnect();
                } else if (status === 'disconnected') {
                    this._documentModels.forEach((model) =>
                        model.onNetworkDisconnected()
                    );
                }
            })
        );

        this.disposeWithMe(
            this._networkService.changesetPushed$.subscribe(async (changeset) => {
                const model = this._documentModels.get(changeset.docId);
                if (model) {
                    this._logger.log(
                        `CollaborationService: Received changeset for ${changeset.docId}, rev ${changeset.serverRev}`
                    );
                    await model.handleRemoteChangeset(
                        changeset.serverRev,
                        changeset.userId
                    );
                }
            })
        );

        this._networkService.connect().catch((error) => {
            this._logger.error('CollaborationService: Failed to connect', error);
        });
    }

    private _initMutationListeners(): void {
        this.disposeWithMe(
            this._commandService.onMutationExecutedForCollab((command, options) => {
                if (options?.fromCollab) return;
                if (command.type !== CommandType.MUTATION) return;

                const params = command.params as { unitId?: string } | undefined;
                const unitId = params?.unitId;
                if (!unitId || isInternalEditorID(unitId)) return;

                const model = this._documentModels.get(unitId);
                if (!model) {
                    this._logger.warn(
                        `CollaborationService: No model for ${unitId}, skipping mutation ${command.id}`
                    );
                    return;
                }

                model.addLocalOperation({
                    id: command.id,
                    type: command.type,
                    params: params as object,
                });

                this._scheduleFlush(unitId);
            })
        );
    }

    private _initInstanceListeners(): void {
        this.disposeWithMe(
            this._univerInstanceService.unitAdded$.subscribe(async (unit) => {
                const unitId = unit.getUnitId();
                if (isInternalEditorID(unitId)) return;
                await this._createDocumentModel(unitId, unit.getRev());
            })
        );

        this.disposeWithMe(
            this._univerInstanceService.unitDisposed$.subscribe((unit) => {
                const unitId = unit.getUnitId();
                if (isInternalEditorID(unitId)) return;
                this._disposeDocumentModel(unitId);
            })
        );
    }

    private _createPersistence(): IDocumentPersistence {
        return {
            savePending: async (docId, mutations, baseRev) => {
                if (!this._pendingMutationService.isReady()) {
                    await new Promise<void>((resolve) => {
                        const subscription = this._pendingMutationService.ready$.subscribe(
                            (ready) => {
                                if (ready) {
                                    subscription.unsubscribe();
                                    resolve();
                                }
                            }
                        );
                    });
                }

                if (mutations.length === 0) {
                    await this._pendingMutationService.clear(docId);
                } else {
                    await this._pendingMutationService.update(docId, mutations, baseRev);
                }
            },
            loadPending: async (docId) => {
                if (!this._pendingMutationService.isReady()) {
                    await new Promise<void>((resolve) => {
                        const subscription = this._pendingMutationService.ready$.subscribe(
                            (ready) => {
                                if (ready) {
                                    subscription.unsubscribe();
                                    resolve();
                                }
                            }
                        );
                    });
                }

                const mutations = this._pendingMutationService.get(docId);
                if (mutations.length === 0) {
                    return null;
                }
                const baseRev = await this._pendingMutationService.getBaseRev(docId);
                return { mutations, baseRev };
            },
            clearPending: async (docId) => {
                await this._pendingMutationService.clear(docId);
            },
        };
    }

    private async _createDocumentModel(
        docId: string,
        initialRev: number
    ): Promise<void> {
        if (this._documentModels.has(docId)) return;

        this._logger.log(
            `CollaborationService: Creating model for ${docId}, rev ${initialRev}`
        );

        const networkOps: IDocumentNetworkOperations = {
            sendChangeset: (docId, mutations, baseRev) =>
                this._networkService.sendChangeset({
                    docId,
                    mutations,
                    baseRev,
                    clientId: this._networkService.getClientId(),
                }),
            fetchOps: (docId, startRev) =>
                this._networkService
                    .fetchOps(docId, startRev)
                    .then((result) => result.operations ?? []),
            joinDoc: (docId) => this._joinDoc(docId),
            leaveDoc: (docId) => this._networkService.leaveDoc(docId),
        };

        const transformOps: IDocumentTransformOperations = {
            transformList: (local, remote) =>
                this._transformService.transformList(local, remote),
            composeList: (mutations) => this._transformService.composeList(mutations),
        };

        const mutationApplier: IDocumentMutationApplier = {
            applyRemoteMutations: async (mutations) => {
                for (const mutation of mutations) {
                    await this._commandService.executeCommand(
                        mutation.id,
                        mutation.params,
                        {
                            fromCollab: true,
                        }
                    );
                }
            },
        };

        const persistence = this._createPersistence();

        const model = new CollaborationDocumentModel(
            docId,
            initialRev,
            networkOps,
            transformOps,
            mutationApplier,
            persistence
        );

        this._documentModels.set(docId, model);

        const stateSubject = new BehaviorSubject<IDocumentSyncState>(
            model.getCurrentState()
        );
        this._documentStateSubjects.set(docId, stateSubject);

        const savedSubject = new BehaviorSubject<boolean>(model.isSaved());
        this._savedStatusSubjects.set(docId, savedSubject);

        this.disposeWithMe(
            model.state$.subscribe((state) => {
                stateSubject.next(state);
                const previousState = this._lastDocumentStates.get(docId);
                this._lastDocumentStates.set(docId, state.state);
                if (state.state === 'pending' && state.pendingCount > 0 && previousState !== 'pending') {
                    this._scheduleFlush(docId);
                }
                // Notify main thread via callback
                if (this._callbackService) {
                    this._callbackService.onDocumentStateChange(docId, state);
                }
            })
        );
        this.disposeWithMe(
            model.saved$.subscribe((saved) => {
                savedSubject.next(saved);
                // Notify main thread via callback
                if (this._callbackService) {
                    this._callbackService.onSavedStatusChange(docId, saved);
                }
            })
        );

        if (this._networkService.isConnected()) {
            await this._joinDoc(docId);
        }
    }

    private async _joinDoc(docId: string): Promise<void> {
        if (this._joinedDocs.has(docId)) {
            this._logger.log(
                `CollaborationService: Already joined ${docId}, skipping`
            );
            return;
        }

        try {
            const ack = await this._networkService.joinDoc(docId);
            if (ack.status === 'ok') {
                this._joinedDocs.add(docId);

                const model = this._documentModels.get(docId);
                const localRev = model?.getServerRev() ?? 0;
                const serverVersion = ack.version ?? 0;

                this._logger.log(
                    `CollaborationService: Joined ${docId}, serverVersion=${serverVersion}, localRev=${localRev}`
                );

                if (model && serverVersion > localRev) {
                    this._logger.log(
                        `CollaborationService: Version gap detected for ${docId}, triggering fetchMiss`
                    );
                    model.onVersionGap();
                }
            }
        } catch (error) {
            this._logger.error(
                `CollaborationService: Failed to join ${docId}`,
                error
            );
        }
    }

    private async _rejoinDocuments(): Promise<void> {
        this._joinedDocs.clear();

        const allUnits: UnitModel[] = [
            ...this._univerInstanceService.getAllUnitsForType<UnitModel>(
                UniverInstanceType.UNIVER_SHEET
            ),
            ...this._univerInstanceService.getAllUnitsForType<UnitModel>(
                UniverInstanceType.UNIVER_DOC
            ),
        ];

        for (const unit of allUnits) {
            const unitId = unit.getUnitId();
            if (isInternalEditorID(unitId)) continue;

            if (!this._documentModels.has(unitId)) {
                await this._createDocumentModel(unitId, unit.getRev());
            } else {
                await this._joinDoc(unitId);
            }
        }
    }

    private _disposeDocumentModel(docId: string): void {
        const model = this._documentModels.get(docId);
        if (model) {
            model.dispose();
            this._documentModels.delete(docId);
        }

        const stateSubject = this._documentStateSubjects.get(docId);
        if (stateSubject) {
            stateSubject.complete();
            this._documentStateSubjects.delete(docId);
        }

        const savedSubject = this._savedStatusSubjects.get(docId);
        if (savedSubject) {
            savedSubject.complete();
            this._savedStatusSubjects.delete(docId);
        }

        const timer = this._flushTimers.get(docId);
        if (timer) {
            clearTimeout(timer);
            this._flushTimers.delete(docId);
        }

        if (this._joinedDocs.has(docId)) {
            this._networkService.leaveDoc(docId);
            this._joinedDocs.delete(docId);
        }

        this._lastDocumentStates.delete(docId);

        this._logger.log(`CollaborationService: Disposed model for ${docId}`);
    }

    private _scheduleFlush(docId: string): void {
        const existingTimer = this._flushTimers.get(docId);
        if (existingTimer) {
            clearTimeout(existingTimer);
        }

        const timer = setTimeout(() => {
            this._flushTimers.delete(docId);
            const model = this._documentModels.get(docId);
            if (model) {
                model.flush();
            }
        }, this._flushDelay);

        this._flushTimers.set(docId, timer);
    }

    /**
     * Flush pending mutations after reconnection
     *
     * When reconnecting after being offline, the state machine goes:
     * offline -> fetchMiss -> pending (if there are pending mutations)
     *
     * This method waits for fetchMiss to complete then triggers flush
     * for all documents with pending mutations.
     */
    private _flushPendingAfterReconnect(): void {
        // Wait for fetchMiss to complete (fetch ops + rejoin docs)
        // The delay accounts for network latency of fetch operations
        const reconnectFlushDelay = 500;

        setTimeout(() => {
            if (!this._networkService.isConnected()) {
                this._logger.log(
                    'CollaborationService: Skipping flush after reconnect - disconnected again'
                );
                return;
            }

            this._documentModels.forEach((model, docId) => {
                const pendingCount = model.getPendingMutations().length;
                const awaitingCount = model.getAwaitingMutations().length;

                if (pendingCount > 0 || awaitingCount > 0) {
                    this._logger.log(
                        `CollaborationService: Flushing ${docId} after reconnect (pending=${pendingCount}, awaiting=${awaitingCount})`
                    );
                    model.flush();
                }
            });
        }, reconnectFlushDelay);
    }

    // ==================== Public API (ICollaborationService) ====================

    getConnectionStatus(): NetworkConnectionStatus {
        return this._connectionStatus$.value;
    }

    getDocumentState(docId: string): IDocumentSyncState {
        const subject = this._documentStateSubjects.get(docId);
        return subject?.value ?? {
            state: 'synced',
            serverRev: 0,
            pendingCount: 0,
            awaitingCount: 0,
        };
    }

    getSavedStatus(docId: string): boolean {
        const subject = this._savedStatusSubjects.get(docId);
        return subject?.value ?? true;
    }

    getDocumentState$(docId: string): Observable<IDocumentSyncState> {
        let subject = this._documentStateSubjects.get(docId);
        if (!subject) {
            subject = new BehaviorSubject<IDocumentSyncState>({
                state: 'synced',
                serverRev: 0,
                pendingCount: 0,
                awaitingCount: 0,
            });
            this._documentStateSubjects.set(docId, subject);
        }
        return subject.asObservable();
    }

    isDocumentSynced(docId: string): boolean {
        const model = this._documentModels.get(docId);
        return model ? model.getCurrentState().state === 'synced' : true;
    }

    getServerRev(docId: string): number {
        const model = this._documentModels.get(docId);
        return model?.getServerRev() ?? 0;
    }

    flush(docId: string): void {
        const model = this._documentModels.get(docId);
        if (model) {
            model.flush();
        }
    }

    reset(docId: string): void {
        const model = this._documentModels.get(docId);
        if (model) {
            model.reset();
        }
    }

    override dispose(): void {
        super.dispose();

        this._documentModels.forEach((model) => model.dispose());
        this._documentModels.clear();

        this._connectionStatus$.complete();
        this._documentStateSubjects.forEach((subject) => subject.complete());
        this._documentStateSubjects.clear();
        this._savedStatusSubjects.forEach((subject) => subject.complete());
        this._savedStatusSubjects.clear();

        this._lastDocumentStates.clear();

        this._flushTimers.forEach((timer) => clearTimeout(timer));
        this._flushTimers.clear();

        this._networkService.disconnect();
    }
}
