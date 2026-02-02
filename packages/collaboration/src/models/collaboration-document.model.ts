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

import type { IDisposable, IMutationInfo } from '@univerjs/core';
import type { Observable } from 'rxjs';
import type { ActorRefFrom } from 'xstate';
import type {
    IChangesetAck,
    IDocumentSyncState,
    IMutationWithOpId,
    IOperationInfo,
    ITransformListResult,
} from '../common/types';
import type { DocumentSyncMachine, IDocumentSyncContext } from './state-machine';
import { Disposable, generateRandomId } from '@univerjs/core';
import { BehaviorSubject } from 'rxjs';
import { createActor } from 'xstate';
import { AsyncLock } from '../common/async-lock';
import { createDocumentSyncMachine } from './state-machine';

/**
 * Interface for network operations (injected dependency)
 */
export interface IDocumentNetworkOperations {
    sendChangeset(docId: string, mutations: IMutationWithOpId[], baseRev: number): Promise<IChangesetAck>;
    fetchOps(docId: string, startRev: number): Promise<IOperationInfo[]>;
    joinDoc(docId: string): Promise<void>;
    leaveDoc(docId: string): void;
}

/**
 * Interface for transform operations (injected dependency)
 */
export interface IDocumentTransformOperations {
    transformList(local: IMutationInfo[], remote: IMutationInfo[]): Promise<ITransformListResult>;
    composeList(mutations: IMutationInfo[]): Promise<IMutationInfo[]>;
}

/**
 * Interface for applying mutations to local state
 */
export interface IDocumentMutationApplier {
    applyRemoteMutations(mutations: IMutationInfo[]): Promise<void>;
}

/**
 * Interface for persisting pending mutations
 */
export interface IDocumentPersistence {
    savePending(docId: string, mutations: IMutationWithOpId[], baseRev: number): Promise<void>;
    loadPending(docId: string): Promise<{ mutations: IMutationWithOpId[]; baseRev: number } | null>;
    clearPending(docId: string): Promise<void>;
}

/**
 * Transform queue item for order preservation
 */
interface ITransformQueueItem {
    id: string;
    type: 'local' | 'remote';
    mutations: IMutationInfo[];
    serverRev?: number;
    userId?: string;
    resolve: (result?: ITransformListResult) => void;
    reject: (error: Error) => void;
}

/**
 * CollaborationDocumentModel - Per-document model managing sync state
 *
 * Responsibilities:
 * - Owns XState actor for sync state management
 * - Manages transform queue for order preservation
 * - Coordinates network, transform, and persistence operations
 * - Provides observables for state monitoring
 */
export class CollaborationDocumentModel extends Disposable implements IDisposable {
    private _actor: ActorRefFrom<DocumentSyncMachine>;

    // Transform queue for order preservation
    private _transformQueue: ITransformQueueItem[] = [];
    private _isProcessingQueue = false;

    // Lock for transform operations to ensure sequential processing
    private readonly _transformLock = new AsyncLock();

    // State observables
    private readonly _state$ = new BehaviorSubject<IDocumentSyncState>({
        state: 'synced',
        serverRev: 0,
        pendingCount: 0,
        awaitingCount: 0,
    });

    private readonly _saved$ = new BehaviorSubject<boolean>(true);

    /** Observable for current sync state */
    readonly state$: Observable<IDocumentSyncState> = this._state$.asObservable();

    /** Observable for saved status (true = no pending mutations) */
    readonly saved$: Observable<boolean> = this._saved$.asObservable();

    constructor(
        /** Document ID */
        readonly docId: string,
        /** Initial server revision */
        private readonly _initialRev: number,
        /** Network operations */
        private readonly _networkOps: IDocumentNetworkOperations,
        /** Transform operations */
        private readonly _transformOps: IDocumentTransformOperations,
        /** Mutation applier */
        private readonly _mutationApplier: IDocumentMutationApplier,
        /** Persistence (optional) */
        private readonly _persistence?: IDocumentPersistence
    ) {
        super();
        this._initStateMachine();
        this._loadPersistedPending();
    }

    /**
     * Initialize the XState actor
     */
    private _initStateMachine(): void {
        const machine = createDocumentSyncMachine({
            // Note: onSendChangeset is now a fire-and-forget callback
            // The result is handled by sending SEND_SUCCESS/SEND_FAILURE events back to the machine
            onSendChangeset: (mutations, baseRev) => {
                this._sendChangesetAsync(mutations, baseRev);
            },
            onFetchOps: (startRev) =>
                this._networkOps.fetchOps(this.docId, startRev),
            onApplyRemote: (mutations) =>
                this._mutationApplier.applyRemoteMutations(mutations),
            onTransform: (local, remote) =>
                this._transformOps.transformList(local, remote),
            onStateChange: (state, context) => {
                this._updateStateObservables(state, context);
            },
            onPersistPending: async (mutations) => {
                if (this._persistence) {
                    const context = this._actor.getSnapshot().context;
                    await this._persistence.savePending(this.docId, mutations, context.serverRev);
                }
            },
        });

        this._actor = createActor(machine, {
            input: {
                docId: this.docId,
                initialRev: this._initialRev,
            },
        });

        // Subscribe to state changes with logging
        this._actor.subscribe((snapshot) => {
            const stateValue = typeof snapshot.value === 'string'
                ? snapshot.value
                : Object.keys(snapshot.value)[0];
            this._updateStateObservables(stateValue, snapshot.context);
        });

        this._actor.start();
    }

    /**
     * Send an event to the state machine with logging
     */
    private _sendEvent(event: Parameters<typeof this._actor.send>[0]): void {
        const eventType = typeof event === 'string' ? event : event.type;
        const snapshot = this._actor.getSnapshot();
        const currentState = typeof snapshot.value === 'string'
            ? snapshot.value
            : Object.keys(snapshot.value)[0];
        console.warn(`[StateMachine:${this.docId}] Sending event: ${eventType} (current state: ${currentState}, serverRev: ${snapshot.context.serverRev})`);
        this._actor.send(event);
        const newSnapshot = this._actor.getSnapshot();
        const newState = typeof newSnapshot.value === 'string'
            ? newSnapshot.value
            : Object.keys(newSnapshot.value)[0];
        console.warn(`[StateMachine:${this.docId}] After event: ${eventType}, new state: ${newState}, serverRev: ${newSnapshot.context.serverRev}`);
    }

    /**
     * Update state observables based on machine state
     */
    private _updateStateObservables(state: string, context: IDocumentSyncContext): void {
        this._state$.next({
            state: state as IDocumentSyncState['state'],
            serverRev: context.serverRev,
            pendingCount: context.pendingMutations.length,
            awaitingCount: context.awaitingMutations.length,
            lastError: context.lastError,
        });

        const isSaved = context.pendingMutations.length === 0 &&
            context.awaitingMutations.length === 0;
        this._saved$.next(isSaved);
    }

    /**
     * Send changeset to server and dispatch result event
     *
     * This is called from the state machine's entry action. The result
     * is dispatched back to the state machine via SEND_SUCCESS/SEND_FAILURE events.
     * This approach allows the network call to complete even if the state changes
     * (e.g., when a LOCAL_OPERATION arrives while awaiting ACK).
     */
    private async _sendChangesetAsync(mutations: IMutationWithOpId[], baseRev: number): Promise<void> {
        try {
            const result = await this._networkOps.sendChangeset(this.docId, mutations, baseRev);
            if (result.status === 'ok') {
                this._sendEvent({
                    type: 'SEND_SUCCESS',
                    serverRev: result.serverRev ?? baseRev + 1,
                });
            } else {
                this._sendEvent({
                    type: 'SEND_FAILURE',
                    error: result.message || 'Changeset rejected by server',
                });
            }
        } catch (error) {
            this._sendEvent({
                type: 'SEND_FAILURE',
                error: String(error),
            });
        }
    }

    /**
     * Load persisted pending mutations on startup
     */
    private async _loadPersistedPending(): Promise<void> {
        if (!this._persistence) return;

        try {
            const persisted = await this._persistence.loadPending(this.docId);
            if (persisted && persisted.mutations.length > 0) {
                this._sendEvent({
                    type: 'LOCAL_OPERATION',
                    mutations: persisted.mutations,
                });
            }
        } catch (error) {
            // Log error but don't fail - we can continue without persisted data
            console.warn(`Failed to load persisted pending mutations for ${this.docId}:`, error);
        }
    }

    /**
     * Add a local operation
     *
     * This method is called when the user makes a local edit.
     * The operation is added to the pending queue and will be sent to the server.
     */
    addLocalOperation(mutation: IMutationInfo): void {
        const mutationWithOpId: IMutationWithOpId = {
            ...mutation,
            opId: generateRandomId(32),
        };

        // Send to state machine
        this._sendEvent({
            type: 'LOCAL_OPERATION',
            mutations: [mutationWithOpId],
        });

        // Persist pending mutations
        this._persistPending();
    }

    /**
     * Add multiple local operations at once
     */
    addLocalOperations(mutations: IMutationInfo[]): void {
        const mutationsWithOpId: IMutationWithOpId[] = mutations.map((m) => ({
            ...m,
            opId: generateRandomId(32),
        }));

        this._sendEvent({
            type: 'LOCAL_OPERATION',
            mutations: mutationsWithOpId,
        });

        this._persistPending();
    }

    /**
     * Handle remote changeset from server
     *
     * This is called when we receive a broadcast from the server about
     * another client's changes. We need to transform our pending operations
     * against the remote operations to maintain consistency.
     */
    async handleRemoteChangeset(serverRev: number, mutations: IMutationInfo[], userId: string): Promise<void> {
        console.warn(`[DocumentModel:${this.docId}] handleRemoteChangeset called: serverRev=${serverRev}, mutations=${mutations.length}, userId=${userId}`);
        // Enqueue for ordered processing
        await this._enqueueAndProcessRemote(serverRev, mutations, userId);
        console.warn(`[DocumentModel:${this.docId}] handleRemoteChangeset complete: serverRev=${serverRev}`);
    }

    /**
     * Enqueue remote operation and process in order
     *
     * This ensures that transforms happen in arrival order, even when
     * transforms are async (e.g., running in a remote context).
     */
    private _enqueueAndProcessRemote(
        serverRev: number,
        mutations: IMutationInfo[],
        userId: string
    ): Promise<ITransformListResult | undefined> {
        return new Promise((resolve, reject) => {
            this._transformQueue.push({
                id: generateRandomId(16),
                type: 'remote',
                mutations,
                serverRev,
                userId,
                resolve,
                reject,
            });
            this._processQueue();
        });
    }

    /**
     * Process the transform queue sequentially
     *
     * This is the key mechanism for preserving operation order despite
     * async transforms. Each operation waits for the previous to complete.
     */
    private async _processQueue(): Promise<void> {
        if (this._isProcessingQueue) {
            console.warn(`[DocumentModel:${this.docId}] Queue already processing, new item will wait`);
            return;
        }
        this._isProcessingQueue = true;
        console.warn(`[DocumentModel:${this.docId}] Starting queue processing, ${this._transformQueue.length} items`);

        try {
            while (this._transformQueue.length > 0) {
                const item = this._transformQueue[0];
                console.warn(`[DocumentModel:${this.docId}] Processing queue item: type=${item.type}, serverRev=${item.serverRev}, remaining=${this._transformQueue.length}`);

                try {
                    if (item.type === 'remote') {
                        await this._processRemoteOperation(item);
                    }
                    item.resolve();
                    console.warn(`[DocumentModel:${this.docId}] Queue item resolved: serverRev=${item.serverRev}`);
                } catch (error) {
                    console.error(`[DocumentModel:${this.docId}] Queue item failed: serverRev=${item.serverRev}`, error);
                    item.reject(error as Error);
                }

                this._transformQueue.shift();
            }
        } finally {
            this._isProcessingQueue = false;
            console.warn(`[DocumentModel:${this.docId}] Queue processing complete`);
        }
    }

    /**
     * Process a remote operation from the queue
     *
     * Uses AsyncLock to ensure transform operations are serialized,
     * even when the underlying transform service is async.
     */
    private async _processRemoteOperation(item: ITransformQueueItem): Promise<void> {
        console.warn(`[DocumentModel:${this.docId}] Processing remote rev ${item.serverRev}, queue size: ${this._transformQueue.length}`);

        // Use lock to ensure transform is atomic
        await this._transformLock.withLock(`remote-rev-${item.serverRev}`, async () => {
            const snapshot = this._actor.getSnapshot();
            const context = snapshot.context;

            console.warn(`[DocumentModel:${this.docId}] Lock acquired for rev ${item.serverRev}, current serverRev: ${context.serverRev}, state: ${typeof snapshot.value === 'string' ? snapshot.value : Object.keys(snapshot.value)[0]}`);

            // Check for version gap
            if (item.serverRev && item.serverRev > context.serverRev + 1) {
                console.warn(`[DocumentModel:${this.docId}] Version gap detected: incoming ${item.serverRev} > expected ${context.serverRev + 1}, triggering FETCH_MISS`);
                this._sendEvent({ type: 'FETCH_MISS' });
                return;
            }

            // Skip if already processed
            if (item.serverRev && item.serverRev <= context.serverRev) {
                console.warn(`[DocumentModel:${this.docId}] Skipping already processed rev ${item.serverRev} (current: ${context.serverRev})`);
                return;
            }

            // Get local mutations that need to be transformed
            const localMutations = [
                ...context.awaitingMutations,
                ...context.pendingMutations,
            ];

            console.warn(`[DocumentModel:${this.docId}] Rev ${item.serverRev}: localMutations=${localMutations.length} (awaiting=${context.awaitingMutations.length}, pending=${context.pendingMutations.length}), remoteMutations=${item.mutations.length}`);

            if (localMutations.length > 0) {
                // Transform local against remote
                console.warn(`[DocumentModel:${this.docId}] Rev ${item.serverRev}: Starting transform...`);
                const result = await this._transformOps.transformList(
                    localMutations,
                    item.mutations
                );
                console.warn(`[DocumentModel:${this.docId}] Rev ${item.serverRev}: Transform complete, error=${result.error}, m1Primes=${result.m1Primes.length}, m2Primes=${result.m2Primes.length}`);

                if (result.error) {
                    this._sendEvent({ type: 'TRANSFORM_ERROR', error: result.error });
                    return;
                }

                // Apply transformed remote mutations (m2Primes)
                if (result.m2Primes.length > 0) {
                    console.warn(`[DocumentModel:${this.docId}] Rev ${item.serverRev}: Applying ${result.m2Primes.length} remote mutations...`);
                    await this._mutationApplier.applyRemoteMutations(result.m2Primes);
                    console.warn(`[DocumentModel:${this.docId}] Rev ${item.serverRev}: Remote mutations applied`);
                }

                // Update state machine with transform result and new server revision
                // awaitingCount tells the state machine how to split m1Primes back into awaiting/pending
                this._sendEvent({
                    type: 'TRANSFORM_COMPLETE',
                    m1Primes: result.m1Primes,
                    m2Primes: result.m2Primes,
                    serverRev: item.serverRev ?? context.serverRev + 1,
                    awaitingCount: context.awaitingMutations.length,
                });
            } else {
                // No local mutations, apply remote directly
                console.warn(`[DocumentModel:${this.docId}] Rev ${item.serverRev}: No local mutations, applying ${item.mutations.length} remote mutations directly...`);
                await this._mutationApplier.applyRemoteMutations(item.mutations);
                console.warn(`[DocumentModel:${this.docId}] Rev ${item.serverRev}: Remote mutations applied, sending RECEIVE_REMOTE`);

                // Update state machine with received remote
                this._sendEvent({
                    type: 'RECEIVE_REMOTE',
                    serverRev: item.serverRev ?? context.serverRev + 1,
                    mutations: item.mutations,
                    userId: item.userId ?? 'unknown',
                });
            }
        });
        console.warn(`[DocumentModel:${this.docId}] Rev ${item.serverRev}: Processing complete, lock released`);
    }

    /**
     * Trigger sending pending mutations to server
     */
    flush(): void {
        const snapshot = this._actor.getSnapshot();
        if (snapshot.context.pendingMutations.length > 0) {
            this._sendEvent({ type: 'SEND_CHANGESET' });
        }
    }

    /**
     * Notify that network is connected
     */
    onNetworkConnected(): void {
        this._sendEvent({ type: 'NETWORK_CONNECTED' });
    }

    /**
     * Notify that network is disconnected
     */
    onNetworkDisconnected(): void {
        this._sendEvent({ type: 'NETWORK_DISCONNECTED' });
    }

    /**
     * Notify that there's a version gap (server is ahead)
     * This triggers fetching missed operations
     */
    onVersionGap(): void {
        this._sendEvent({ type: 'FETCH_MISS' });
    }

    /**
     * Resolve a conflict state
     */
    resolveConflict(): void {
        this._sendEvent({ type: 'RESOLVE_CONFLICT' });
    }

    /**
     * Reset to clean state (discards pending)
     */
    reset(): void {
        this._sendEvent({ type: 'RESET' });
        this._persistence?.clearPending(this.docId);
    }

    /**
     * Get current pending mutations
     */
    getPendingMutations(): IMutationWithOpId[] {
        return this._actor.getSnapshot().context.pendingMutations;
    }

    /**
     * Get current awaiting mutations
     */
    getAwaitingMutations(): IMutationWithOpId[] {
        return this._actor.getSnapshot().context.awaitingMutations;
    }

    /**
     * Get current server revision
     */
    getServerRev(): number {
        return this._actor.getSnapshot().context.serverRev;
    }

    /**
     * Get current sync state
     */
    getCurrentState(): IDocumentSyncState {
        return this._state$.getValue();
    }

    /**
     * Check if document is saved (no pending operations)
     */
    isSaved(): boolean {
        return this._saved$.getValue();
    }

    /**
     * Set local user's awareness state
     *
     * This stores the awareness state in the document model.
     * The actual network broadcast is handled by CollaborationService.
     *
     * @param awareness User awareness state (selection, cursor, etc.)
     */
    setLocalAwareness(awareness: import('../common/types').IUserAwareness): void {
        // For now, we just store it in context for potential future use
        // The actual broadcast is done by CollaborationService via NetworkService
        // This method can be extended to:
        // - Store awareness in document context
        // - Track awareness history
        // - Throttle awareness updates

        console.warn(`[CollaborationDocumentModel] setLocalAwareness for ${this.docId}:`, awareness);
    }

    /**
     * Persist pending mutations
     */
    private async _persistPending(): Promise<void> {
        if (!this._persistence) return;

        const context = this._actor.getSnapshot().context;
        await this._persistence.savePending(
            this.docId,
            context.pendingMutations,
            context.serverRev
        );
    }

    override dispose(): void {
        super.dispose();
        this._actor.stop();
        this._state$.complete();
        this._saved$.complete();
        this._transformQueue = [];
    }
}
