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

import type { IMutationInfo } from '@univerjs/core';
import type { IChangeset, IChangesetAck, IChangesetRequest, IFetchOpsAck, IFetchOpsRequest, IJoinDocAck, IJoinDocRequest, IOperationInfo } from './socket.service';
import { createIdentifier, Disposable, generateRandomId, ILogService } from '@univerjs/core';
import { IOfflineStorageService } from './offline-storage.service';
import { ISocketService } from './socket.service';

export interface ICollaborationService {
    sendChangeset(changeset: IChangeset): void;
    joinDoc(docId: string): void;
    leaveDoc(docId: string): void;
    flush(unitId?: string): Promise<void>;
    fetchOps(docId: string, startRev: number): Promise<IOperationInfo[]>;
    syncOnReconnect(unitId: string): Promise<{ missedOps: IOperationInfo[]; pendingMutations: IMutationInfo[]; serverVersion: number }>;
    getPendingMutations(unitId: string): IMutationInfo[];
    setTransformedPendingMutations(unitId: string, mutations: IMutationInfo[], baseRev: number): void;
    getCurrentVersion(unitId: string): number | undefined;
    setCurrentVersion(unitId: string, version: number): void;
}

export const ICollaborationService = createIdentifier<ICollaborationService>('univer.collaboration.service');

export class CollaborationService extends Disposable implements ICollaborationService {
    private _joinedDocs: Set<string> = new Set();
    private _currentVersions: Map<string, number> = new Map();

    // 按 unitId 积攒的 mutations 队列
    private _pendingMutations: Map<string, IMutationInfo[]> = new Map();

    // 每个 unitId 的 debounce 定时器
    private _debounceTimers: Map<string, NodeJS.Timeout> = new Map();

    // 每个 unitId 的 baseRev（用于批次发送）
    private _pendingBaseRevs: Map<string, number> = new Map();

    // 每个 unitId 的 userId（用于批次发送）
    private _pendingUserIds: Map<string, string> = new Map();

    // 批次大小限制
    private readonly _BATCH_SIZE_LIMIT = 20;

    // Debounce 延迟时间（毫秒）
    private readonly _DEBOUNCE_DELAY = 200;

    constructor(
        @ISocketService private readonly _socketService: ISocketService,
        @ILogService private readonly _logger: ILogService,
        @IOfflineStorageService private readonly _offlineStorage: IOfflineStorageService
    ) {
        super();
        this._loadPendingMutationsFromStorage();
        this._initSocketListeners();
    }

    /**
     * Initialize socket connection/disconnection listeners
     */
    private _initSocketListeners(): void {
        // Clear joined docs on disconnect so we rejoin on reconnect
        this.disposeWithMe(this._socketService.disconnected$.subscribe(() => {
            this._logger.log('Socket disconnected, clearing joined docs for rejoin on reconnect');
            this._joinedDocs.clear();
            // Don't clear currentVersions - we need them to detect version gaps on reconnect
        }));
    }

    /**
     * Load pending mutations from offline storage on initialization
     */
    private async _loadPendingMutationsFromStorage(): Promise<void> {
        try {
            const allPending = await this._offlineStorage.loadAllPendingMutations();
            for (const pending of allPending) {
                this._pendingMutations.set(pending.unitId, pending.mutations);
                this._pendingBaseRevs.set(pending.unitId, pending.baseRev);
                this._pendingUserIds.set(pending.unitId, pending.userId);
                this._logger.log(`Loaded ${pending.mutations.length} pending mutations for unitId: ${pending.unitId}`);
            }
        } catch (error) {
            this._logger.error('Failed to load pending mutations from storage:', error);
        }
    }

    joinDoc(docId: string): Promise<void> {
        if (!this._socketService.getSocket() || this._socketService.getSocket()?.disconnected) {
            this._logger.error('Socket not connected');
            return Promise.reject(new Error('Socket not connected'));
        }

        if (this._joinedDocs.has(docId)) {
            this._logger.log(`Already joined doc: ${docId}`);
            return Promise.resolve();
        }

        const request: IJoinDocRequest = { docId };
        return new Promise<void>((resolve, reject) => {
            this._socketService.emit('join_doc', request, (ack: IJoinDocAck) => {
                if (ack.status === 'ok') {
                    this._joinedDocs.add(docId);
                    if (ack.version !== undefined) {
                        this._currentVersions.set(docId, ack.version);
                    }
                    this._logger.log(`Joined doc ${docId}, version: ${ack.version}`);
                    resolve();

              // TODO: Load content from ack.content if needed
                } else {
                    this._logger.error(`Failed to join doc ${docId}: ${ack.message}`);
                    reject(new Error(`Failed to join doc ${docId}: ${ack.message}`));
                }
            });
        });
    }

    leaveDoc(docId: string): void {
        if (!this._socketService.getSocket() || this._socketService.getSocket()?.disconnected) {
            return;
        }

        if (!this._joinedDocs.has(docId)) {
            return;
        }

        this._socketService.emit('leave_doc', { docId });
        this._joinedDocs.delete(docId);
        this._currentVersions.delete(docId);
        this._logger.log(`Left doc: ${docId}`);
    }

    async sendChangeset(changeset: IChangeset): Promise<void> {
        if (!this._socketService.getSocket() || this._socketService.getSocket()?.disconnected) {
            this._logger.error('Socket not connected, saving to offline storage');
            // Save to offline storage when socket is disconnected
            const unitId = changeset.unitId;
            if (unitId) {
                try {
                    await this._offlineStorage.savePendingMutations(
                        unitId,
                        changeset.mutations,
                        changeset.baseRev,
                        changeset.userId
                    );
                } catch (error) {
                    this._logger.error('Failed to save to offline storage:', error);
                }
            }
            return Promise.reject(new Error('Socket not connected'));
        }

        // Extract unitId from mutations (unitId is used as docId)
        const unitId = changeset.unitId;
        if (!unitId) {
            this._logger.error('Cannot determine unitId from changeset');
            return Promise.reject(new Error('Cannot determine unitId from changeset'));
        }

        // Ensure joined to doc
        if (!this._joinedDocs.has(unitId)) {
            this._logger.warn(`Not joined to doc: ${unitId}`);
            await this.joinDoc(unitId);
        }

        // Initialize queue if not exists
        if (!this._pendingMutations.has(unitId)) {
            this._pendingMutations.set(unitId, []);
        }

        // Add mutations to queue
        const queue = this._pendingMutations.get(unitId)!;
        queue.push(...changeset.mutations);

        // Record baseRev and userId on first mutation
        if (!this._pendingBaseRevs.has(unitId)) {
            const currentVersion = this._currentVersions.get(unitId);
            this._pendingBaseRevs.set(unitId, currentVersion ?? changeset.baseRev);
            this._pendingUserIds.set(unitId, changeset.userId);
        }

        // Check if reached batch size limit
        if (queue.length >= this._BATCH_SIZE_LIMIT) {
            // Clear debounce timer and flush immediately
            this._clearDebounceTimer(unitId);
            await this._flushChangeset(unitId);
        } else {
            // Set or reset debounce timer
            this._setDebounceTimer(unitId);
        }
    }

    /**
     * Clear debounce timer for a specific unitId
     */
    private _clearDebounceTimer(unitId: string): void {
        const timer = this._debounceTimers.get(unitId);
        if (timer) {
            clearTimeout(timer);
            this._debounceTimers.delete(unitId);
        }
    }

    /**
     * Set or reset debounce timer for a specific unitId
     */
    private _setDebounceTimer(unitId: string): void {
        // Clear existing timer
        this._clearDebounceTimer(unitId);

        // Set new timer
        const timer = setTimeout(async () => {
            await this._flushChangeset(unitId);
        }, this._DEBOUNCE_DELAY);

        this._debounceTimers.set(unitId, timer);
    }

    /**
     * Flush pending mutations for a specific unitId
     */
    private async _flushChangeset(unitId: string): Promise<void> {
        // Clear debounce timer
        this._clearDebounceTimer(unitId);

        // Get pending mutations, baseRev, and userId
        const mutations = this._pendingMutations.get(unitId);
        const baseRev = this._pendingBaseRevs.get(unitId);
        const userId = this._pendingUserIds.get(unitId);

        // If no pending mutations, return
        if (!mutations || mutations.length === 0) {
            return;
        }

        // If no baseRev, use current version or 0
        const currentBaseRev = baseRev ?? this._currentVersions.get(unitId) ?? 0;

        // If no userId, error
        if (!userId) {
            this._logger.error('Cannot determine userId for unitId');
            // Put mutations back to queue for retry
            this._pendingMutations.set(unitId, mutations);
            this._pendingBaseRevs.set(unitId, currentBaseRev);
            return;
        }

        // Clear queue, baseRev, and userId before sending
        this._pendingMutations.delete(unitId);
        this._pendingBaseRevs.delete(unitId);
        this._pendingUserIds.delete(unitId);

        // Build request
        const clientMsgId = `${this._socketService.getSocket()?.id}-${Date.now()}-${generateRandomId()}`;
        const request: IChangesetRequest = {
            baseRev: currentBaseRev,
            clientMsgId,
            mutations,
            userId,
            docId: unitId,
        };

        // Send to server
        return new Promise<void>((resolve, reject) => {
            this._socketService.emit('changeset', request, async (ack: IChangesetAck) => {
                if (ack.status === 'ok' && ack.serverRev !== undefined) {
                    this._currentVersions.set(unitId, ack.serverRev);
                    this._logger.log(`Changeset applied, new version: ${ack.serverRev}, mutations count: ${mutations.length}`);
                    // Clear offline storage for this unitId since mutations were successfully sent
                    try {
                        await this._offlineStorage.clearPendingMutations(unitId);
                    } catch (error) {
                        this._logger.error('Failed to clear offline storage:', error);
                    }
                    resolve();
                } else {
                    this._logger.error(`Changeset failed: ${ack.message}`);
                    // Put mutations back to queue for retry
                    const existingMutations = this._pendingMutations.get(unitId) || [];
                    this._pendingMutations.set(unitId, [...existingMutations, ...mutations]);
                    this._pendingBaseRevs.set(unitId, currentBaseRev);
                    this._pendingUserIds.set(unitId, userId);
                    // Save to offline storage for retry later
                    try {
                        await this._offlineStorage.savePendingMutations(
                            unitId,
                            [...existingMutations, ...mutations],
                            currentBaseRev,
                            userId
                        );
                    } catch (error) {
                        this._logger.error('Failed to save to offline storage:', error);
                    }
                    // TODO: Handle version mismatch - may need to fetch_ops and resync
                    reject(new Error(`Changeset failed: ${ack.message}`));
                }
            });
        });
    }

    /**
     * Fetch operations from server since startRev
     */
    async fetchOps(docId: string, startRev: number): Promise<IOperationInfo[]> {
        if (!this._socketService.getSocket() || this._socketService.getSocket()?.disconnected) {
            this._logger.error('Socket not connected');
            return Promise.reject(new Error('Socket not connected'));
        }

        const request: IFetchOpsRequest = { docId, startRev };
        return new Promise<IOperationInfo[]>((resolve, reject) => {
            this._socketService.emit('fetch_ops', request, (ack: IFetchOpsAck) => {
                if (ack.status === 'ok' && ack.operations) {
                    this._logger.log(`Fetched ${ack.operations.length} operations for doc ${docId} since rev ${startRev}`);
                    resolve(ack.operations);
                } else {
                    this._logger.error(`Failed to fetch ops: ${ack.message}`);
                    reject(new Error(`Failed to fetch ops: ${ack.message}`));
                }
            });
        });
    }

    /**
     * Sync on reconnect: fetch missed operations and return them for processing
     * The controller will handle applying mutations and transforming pending ops
     */
    async syncOnReconnect(unitId: string): Promise<{ missedOps: IOperationInfo[]; pendingMutations: IMutationInfo[]; serverVersion: number }> {
        if (!this._socketService.getSocket() || this._socketService.getSocket()?.disconnected) {
            this._logger.error('Socket not connected');
            return Promise.reject(new Error('Socket not connected'));
        }

        // Get local version (from currentVersions or pending baseRev)
        const localVersion = this._currentVersions.get(unitId) ?? this._pendingBaseRevs.get(unitId) ?? 0;

        // Join doc to get server version
        try {
            await this.joinDoc(unitId);
        } catch (error) {
            this._logger.error(`Failed to join doc ${unitId} during sync:`, error);
            throw error;
        }

        const serverVersion = this._currentVersions.get(unitId);
        if (serverVersion === undefined) {
            this._logger.error(`Server version not available for unitId: ${unitId}`);
            return { missedOps: [], pendingMutations: [], serverVersion: 0 };
        }

        // Get pending mutations (from memory and offline storage)
        let pendingMutations = this._pendingMutations.get(unitId) || [];

        // Also load from offline storage in case there are mutations saved there
        try {
            const offlinePending = await this._offlineStorage.loadPendingMutations(unitId);
            if (offlinePending && offlinePending.mutations.length > 0) {
                // Merge offline mutations if not already in memory
                if (pendingMutations.length === 0) {
                    pendingMutations = offlinePending.mutations;
                    this._pendingMutations.set(unitId, pendingMutations);
                    this._pendingBaseRevs.set(unitId, offlinePending.baseRev);
                    this._pendingUserIds.set(unitId, offlinePending.userId);
                }
            }
        } catch (error) {
            this._logger.error('Failed to load offline pending mutations:', error);
        }

        // If local version is behind server, fetch missed operations
        let missedOps: IOperationInfo[] = [];
        if (localVersion < serverVersion) {
            this._logger.log(`Version gap detected: local=${localVersion}, server=${serverVersion}, fetching missed operations`);
            missedOps = await this.fetchOps(unitId, localVersion);

            // Update baseRev for pending mutations to reflect server version
            if (pendingMutations.length > 0) {
                this._pendingBaseRevs.set(unitId, serverVersion);
            }
        }

        return { missedOps, pendingMutations, serverVersion };
    }

    /**
     * Get pending mutations for a unit
     */
    getPendingMutations(unitId: string): IMutationInfo[] {
        return this._pendingMutations.get(unitId) || [];
    }

    /**
     * Set transformed pending mutations after OT
     */
    setTransformedPendingMutations(unitId: string, mutations: IMutationInfo[], baseRev: number): void {
        this._pendingMutations.set(unitId, mutations);
        this._pendingBaseRevs.set(unitId, baseRev);
    }

    /**
     * Get the latest known server version for a unit
     */
    getCurrentVersion(unitId: string): number | undefined {
        return this._currentVersions.get(unitId);
    }

    /**
     * Update the latest known server version for a unit
     */
    setCurrentVersion(unitId: string, version: number): void {
        this._currentVersions.set(unitId, version);
    }

    /**
     * Flush pending mutations immediately
     * @param unitId If provided, only flush this unitId's queue. Otherwise flush all.
     */
    async flush(unitId?: string): Promise<void> {
        if (unitId) {
            // Flush specific unitId
            await this._flushChangeset(unitId);
        } else {
            // Flush all unitIds
            const unitIds = Array.from(this._pendingMutations.keys());
            await Promise.all(unitIds.map((id) => this._flushChangeset(id)));
        }
    }

    override dispose(): void {
        // Clear all debounce timers
        for (const unitId of this._debounceTimers.keys()) {
            this._clearDebounceTimer(unitId);
        }

        // Flush all pending mutations
        const unitIds = Array.from(this._pendingMutations.keys());
        for (const unitId of unitIds) {
            // Flush synchronously to ensure all mutations are sent before dispose
            this._flushChangeset(unitId).catch((error) => {
                this._logger.error(`Error flushing changeset for ${unitId} during dispose:`, error);
            });
        }

        super.dispose();
        // Leave all joined docs
        for (const docId of this._joinedDocs) {
            this.leaveDoc(docId);
        }
    }
}
