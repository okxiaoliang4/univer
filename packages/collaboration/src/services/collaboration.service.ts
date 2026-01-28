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

import type { Observable, Subscription } from 'rxjs';
import type { IMutationWithOpId } from './collaboration.types';
import type {
    IChangeset,
    IChangesetAck,
    IChangesetRequest,
    IFetchOpsAck,
    IFetchOpsRequest,
    IJoinDocAck,
    IJoinDocRequest,
    IOperationInfo,
} from './socket.service';
import {
    createIdentifier,
    Disposable,
    generateRandomId,
    ICommandService,
    IConfigService,
    ILogService,
    isInternalEditorID,
    IUniverInstanceService,
    sequenceExecute,
} from '@univerjs/core';
import { BehaviorSubject, Subject } from 'rxjs';
import { IPendingMutationSerivce } from './offline-storage.service';
import { ISocketService } from './socket.service';
import { ITransformService } from './transform.service';

export interface ICollaborationService {
    ready$: Observable<string>;
    connectStatus$: Observable<'connected' | 'disconnected' | 'connecting'>;
    saved$: Record<string, BehaviorSubject<boolean>>;
    getSavedStatus(unitId: string): BehaviorSubject<boolean>;
    sendChangeset(changeset: IChangeset): void;
    joinDoc(docId: string): void;
    leaveDoc(docId: string): void;
    flush(unitId?: string): Promise<void>;
    fetchOps(docId: string, startRev: number): Promise<IOperationInfo[]>;
    getDocRev(docId: string): number;
    updateDocRev(docId: string, rev: number): void;
    docJoined$: Observable<string>;
    docLeft$: Observable<string>;
}

export const ICollaborationService = createIdentifier<ICollaborationService>(
    'univer.collaboration.service'
);

export class CollaborationService
    extends Disposable
    implements ICollaborationService {
    private _joinedDocs: Set<string> = new Set();

    // 每个 unitId 的 debounce 定时器
    private _debounceTimers: Map<string, NodeJS.Timeout> = new Map();
    private _inFlightFlushes: Set<string> = new Set();

    // 批次大小限制
    private readonly _batchSizeLimit = 20;

    // Debounce 延迟时间（毫秒）
    private readonly _debounceDelay = 0;

    private _collabUnits: Set<string> = new Set();
    private _pendingJoins: Map<
        string,
        { promise: Promise<void>; resolve: () => void; subscription: Subscription }
    > = new Map();

    private _docJoined$ = new Subject<string>();
    private _docLeft$ = new Subject<string>();

    docJoined$ = this._docJoined$.asObservable();
    docLeft$ = this._docLeft$.asObservable();

    // ready$ 代表哪个 doc 已经 joinDoc 了
    private _ready$ = new Subject<string>();
    ready$ = this._ready$.asObservable();

    // connectStatus$ 代表网络状态
    private _connectStatus$ = new BehaviorSubject<'connected' | 'disconnected' | 'connecting'>('disconnected');
    connectStatus$ = this._connectStatus$.asObservable();

    // saved$ 代表每个 doc 的保存状态，true 表示已保存，false 表示有未保存的操作
    private _savedSubjects: Record<string, BehaviorSubject<boolean>> = {};
    saved$ = this._savedSubjects;

    constructor(
        @ISocketService private readonly _socketService: ISocketService,
        @ILogService private readonly _logger: ILogService,
        @IPendingMutationSerivce private readonly _pendingMutationService: IPendingMutationSerivce,
        @IUniverInstanceService private readonly _univerInstanceService: IUniverInstanceService,
        @ICommandService private readonly _commandService: ICommandService,
        @ITransformService private readonly _transformService: ITransformService,
        @IConfigService private readonly _configService: IConfigService
    ) {
        super();
        this._init();
    }

    private _init(): void {
        this._initSocketListeners();
        this._initInstanceListener();
        this._initChangesetPushedListener();
    }

    /**
     * Initialize socket connection/disconnection listeners
     */
    private _initSocketListeners(): void {
        // Update initial connection status based on current socket state
        this._updateConnectStatus();

        // Listen for socket connection status changes
        const socket = this._socketService.getSocket();
        if (socket) {
            // Listen for connecting event (fires when socket starts connecting)
            socket.on('connecting', () => {
                this._connectStatus$.next('connecting');
            });
        }

        this.disposeWithMe(
            this._socketService.connected$.subscribe(() => {
                this._connectStatus$.next('connected');
                this._collabUnits.forEach(async (unitId) => {
                    await this.joinDoc(unitId);
                });
            })
        );

        // Clear joined docs on disconnect so we rejoin on reconnect
        this.disposeWithMe(
            this._socketService.disconnected$.subscribe(() => {
                this._connectStatus$.next('disconnected');
                this._logger.log(
                    'Socket disconnected, clearing joined docs for rejoin on reconnect'
                );
                this._joinedDocs.clear();
                this._collabUnits.forEach(async (unitId) => {
                    await this.leaveDoc(unitId);
                });
            })
        );
    }

    /**
     * Update connection status based on current socket state
     */
    private _updateConnectStatus(): void {
        const socket = this._socketService.getSocket();
        if (!socket) {
            this._connectStatus$.next('disconnected');
            return;
        }

        if (socket.connected) {
            this._connectStatus$.next('connected');
        } else if (socket.disconnected) {
            this._connectStatus$.next('disconnected');
        } else {
            // Socket exists but not connected yet (connecting state)
            // This happens when socket is created but connection is in progress
            this._connectStatus$.next('connecting');
        }
    }

    /**
     * Get saved status BehaviorSubject for a specific unitId
     * Creates a new BehaviorSubject if it doesn't exist
     */
    getSavedStatus(unitId: string): BehaviorSubject<boolean> {
        if (!this._savedSubjects[unitId]) {
            // 初始化时检查是否有 pending mutations
            const pendingMutations = this._pendingMutationService.get(unitId);
            const initialStatus = !pendingMutations || pendingMutations.length === 0;
            this._savedSubjects[unitId] = new BehaviorSubject<boolean>(initialStatus);
        }
        return this._savedSubjects[unitId];
    }

    /**
     * Update saved status for a specific docId
     */
    private _updateSavedStatus(docId: string): void {
        const pendingMutations = this._pendingMutationService.get(docId);
        const isSaved = !pendingMutations || pendingMutations.length === 0;
        const subject = this.getSavedStatus(docId);
        subject.next(isSaved);
    }

    private _initInstanceListener(): void {
        this.disposeWithMe(
            this._univerInstanceService.unitAdded$.subscribe(async (unit) => {
                const unitId = unit.getUnitId();
                if (isInternalEditorID(unitId)) return;
                this._collabUnits.add(unitId);
                await this.joinDoc(unitId);
            })
        );

        this.disposeWithMe(
            this._univerInstanceService.unitDisposed$.subscribe((workbook) => {
                const unitId = workbook.getUnitId();
                if (isInternalEditorID(unitId)) return;
                this._collabUnits.delete(unitId);
                this.leaveDoc(unitId);
            })
        );
    }

    /**
     * Handle changeset_pushed event from server (OT broadcast)
     *
     * This implements the client-side OT algorithm using "coordinate realignment" (rebase) strategy:
     * - When receiving a broadcast while having pending mutations, we DON'T undo local operations
     * - Instead, we transform both the pending and incoming operations to maintain consistency
     * - This avoids UI flickering and provides a smooth collaborative editing experience
     *
     * Example scenario:
     * 1. Client B sends mutation m_B (pending), local UI shows: $State + m_B
     * 2. Client B receives broadcast(m_A) from server
     * 3. Transform: (m_B', m_A') = transform(m_B, m_A)
     * 4. Apply m_A' to current UI (which already has m_B applied)
     * 5. Update pending to m_B' (coordinate realignment)
     * 6. When Ack arrives, clear pending
     * Final state: $State + m_B + m_A' (guaranteed to equal $State + m_A + m_B' by OT properties)
     */
    // eslint-disable-next-line max-lines-per-function
    private _initChangesetPushedListener(): void {
        this.disposeWithMe(
            // eslint-disable-next-line max-lines-per-function
            this._socketService.changesetPushed$.subscribe(async (changeset) => {
                const unitId = changeset.docId;
                const unit = this._univerInstanceService.getUnit(unitId);

                if (!unit || isInternalEditorID(unitId)) {
                    this._logger.warn(`Unit not found or is internal: ${unitId}`);
                    return;
                }

                const localRev = unit.getRev();
                const expectedRev = localRev + 1;
                const serverRev = changeset.serverRev;
                const remoteMutations = changeset.mutations;

                if (serverRev <= localRev) return; // 已经处理过的版本，直接忽略
                // Check for version gap - need to fetch missed operations
                if (serverRev > expectedRev) {
                    this._logger.warn(
                        `changeset_pushed gap: unit=${unitId}, localRev=${localRev}, expectedRev=${expectedRev}, serverRev=${serverRev}`
                    );
                    await this._resync(unitId, localRev);
                } else {
                    // Normal case: no version gap
                    const pendingMutations =
                        await this._pendingMutationService.get(unitId);

                    this._logger.log(
                        `changeset_pushed: unit=${unitId}, localRev=${localRev}, serverRev=${serverRev}, pendingLen=${pendingMutations?.length ?? 0}`
                    );

                    if (pendingMutations?.length && pendingMutations.length > 0) {
                        // OT Coordinate Realignment Strategy:
                        // 1. Transform pending and remote mutations against each other
                        // 2. Apply transformed remote mutations (m2Primes) to current UI
                        // 3. Update pending queue with transformed versions (m1Primes)
                        // This ensures: $State + m1 + m2' = $State + m2 + m1' (OT consistency)
                        const result = this._transformService.transformList(
                            pendingMutations ?? [],
                            remoteMutations
                        );
                        if (result.error) {
                            this._logger.error(`Transform error: ${result.error}`);
                            const filteredRemoteMutations = remoteMutations;
                            if (filteredRemoteMutations.length > 0) {
                                await sequenceExecute(
                                    filteredRemoteMutations,
                                    this._commandService,
                                    { fromCollab: true }
                                );
                            }
                        } else {
                            this._logger.log(
                                `OT result: m1Primes=${result.m1Primes.length}, m2Primes=${result.m2Primes.length}`
                            );

                            // Apply transformed server mutations (m2') to current UI
                            // Note: Current UI already has pending mutations applied
                            const filteredM2Primes = result.m2Primes;
                            if (filteredM2Primes.length > 0) {
                                await sequenceExecute(filteredM2Primes, this._commandService, {
                                    fromCollab: true,
                                });
                            }

                            // Update pending queue with transformed versions (m1')
                            // This "coordinate realignment" ensures future operations are correctly positioned
                            const nextPending: IMutationWithOpId[] = result.m1Primes.map((mutation) => ({
                                ...mutation,
                                opId: generateRandomId(32),
                            }));
                            await this._pendingMutationService.update(
                                unitId,
                                nextPending,
                                serverRev
                            );
                        }
                    } else {
                        // No pending mutations, apply directly
                        const filteredRemoteMutations = remoteMutations;
                        if (filteredRemoteMutations.length > 0) {
                            const result = await sequenceExecute(
                                filteredRemoteMutations,
                                this._commandService,
                                { fromCollab: true }
                            );
                            this._logger.log(`sequenceExecute result: ${result}`);
                        }
                    }
                    this.updateDocRev(unitId, serverRev);
                }
            })
        );
    }

    async joinDoc(docId: string): Promise<void> {
        if (
            !this._socketService.getSocket() ||
            this._socketService.getSocket()?.disconnected
        ) {
            const pending = this._pendingJoins.get(docId);
            if (pending) {
                return pending.promise;
            }

            let resolveFn: () => void;
            const promise = new Promise<void>((resolve) => {
                resolveFn = resolve;
            });
            const subscription = this._socketService.connected$.subscribe(() => {
                const entry = this._pendingJoins.get(docId);
                if (!entry) return;
                entry.subscription.unsubscribe();
                this._pendingJoins.delete(docId);
                this._joinDocNow(docId).finally(entry.resolve);
            });
            this._pendingJoins.set(docId, {
                promise,
                resolve: resolveFn!,
                subscription,
            });
            return promise;
        }

        return this._joinDocNow(docId);
    }

    private async _joinDocNow(docId: string): Promise<void> {
        if (this._joinedDocs.has(docId)) {
            this._logger.log(`Already joined doc: ${docId}`);
            // 如果已经加入，也发出 ready$ 事件
            this._ready$.next(docId);
            return;
        }

        const request: IJoinDocRequest = { docId };
        await new Promise<void>((resolve) => {
            this._socketService.emit('join_doc', request, (ack: IJoinDocAck) => {
                if (ack.status === 'ok') {
                    this._joinedDocs.add(docId);
                    this._logger.log(`Joined doc ${docId}, version: ${ack.version}`);
                    this._docJoined$.next(docId);
                    // 发出 ready$ 事件，表示该 doc 已经成功 joinDoc
                    this._ready$.next(docId);
                    // 初始化保存状态
                    this._updateSavedStatus(docId);
                    resolve();
                }
            });
        });
        await this._resync(docId, this.getDocRev(docId));
    }

    /**
     * Sync version after joining a doc - fetch missed ops if local version is behind
     */
    private async _resync(unitId: string, localRev: number): Promise<void> {
        try {
            // 1. 获取本地离线期间产生且尚未被服务器 Ack 的操作 (m1_list)
            // 注意：你需要从你的客户端状态管理器中取出 pending 队列或 buffer 队列
            const pendingMutations = this._pendingMutationService.get(unitId) ?? [];
            const pendingWithOpId: IMutationWithOpId[] = pendingMutations.map((mutation) => ({
                ...mutation,
                opId: mutation.opId || generateRandomId(32),
            }));
            // 2. 获取服务器上从 localRev 开始的所有后续操作 (m2_list)
            const missedOps = await this.fetchOps(unitId, localRev);

            this._logger.log(
                `resync: unit=${unitId}, localRev=${localRev}, pendingLen=${pendingMutations.length}, missedOpsLen=${missedOps.length}, missedOpsRevs=${missedOps.map((op) => op.rev).join(',')}`
            );

            if (missedOps.length > 0) {
                // 3. 执行 OT 转换
                // m1: 本地离线, m2: 服务器历史
                const result = await this._transformService.transformList(
                    pendingWithOpId,
                    [...missedOps.flatMap((op) => op.mutations)]
                );

                if (result.error) {
                    this._logger.error(`Transform error: ${result.error}`);
                    // 严重错误：可能需要强制刷新页面重新拉取快照
                    return;
                }

                this._logger.log(
                    `resync transform: unit=${unitId}, m1Primes=${result.m1Primes.map((m) => m.id).join(',')}, m2Primes=${result.m2Primes.map((m) => m.id).join(',')}`
                );

                // 4. 应用 m2Primes 到本地 UI
                // 目的：让本地看到别人在你离线时做的修改（已针对你的修改做过偏移）
                const filteredM2Primes = result.m2Primes;
                if (filteredM2Primes.length > 0) {
                    await sequenceExecute(filteredM2Primes, this._commandService, {
                        fromCollab: true,
                    });
                }

                // 5. 更新本地版本号
                // 版本号应直接对齐到服务器已知的最新版本
                const serverLatestRev = localRev + missedOps.length;
                this.updateDocRev(unitId, serverLatestRev);

                // 6. 处理 m1Primes (你的离线操作经过偏移后的新样子)
                if (result.m1Primes.length > 0) {
                    const nextPending: IMutationWithOpId[] = result.m1Primes.map((mutation) => ({
                        ...mutation,
                        opId: generateRandomId(32),
                    }));
                    this._pendingMutationService.update(
                        unitId,
                        nextPending,
                        serverLatestRev
                    );
                    // 更新保存状态：有未保存的操作
                    this._updateSavedStatus(unitId);
                    await this._flushChangeset(unitId);
                } else {
                    // 如果没有 pending mutations，更新为已保存
                    this._updateSavedStatus(unitId);
                }

                this._logger.log(
                    `Sync completed. Applied ${missedOps.length} remote ops. Sent ${result.m1Primes.length} local ops.`
                );
            } else if (pendingMutations.length > 0) {
                // 如果服务器没新东西，但本地有离线操作，直接补发即可
                // 更新保存状态：有未保存的操作
                this._updateSavedStatus(unitId);
                await this.flush(unitId);
            } else {
                // 如果没有 pending mutations，更新为已保存
                this._updateSavedStatus(unitId);
            }
        } catch (error) {
            this._logger.error(`Failed to fetch ops for ${unitId}:`, error);
        }
    }

    getDocRev(docId: string): number {
        return this._univerInstanceService.getUnit(docId)?.getRev() ?? 0;
    }

    updateDocRev(docId: string, rev: number): void {
        this._univerInstanceService.getUnit(docId)?.setRev(rev);
    }

    leaveDoc(docId: string): void {
        const pending = this._pendingJoins.get(docId);
        if (pending) {
            pending.subscription.unsubscribe();
            pending.resolve();
            this._pendingJoins.delete(docId);
        }

        this._joinedDocs.delete(docId);
        this._docLeft$.next(docId);

        // 清除该文档的保存状态 BehaviorSubject
        if (this._savedSubjects[docId]) {
            this._savedSubjects[docId].complete();
            delete this._savedSubjects[docId];
        }

        if (
            !this._socketService.getSocket() ||
            this._socketService.getSocket()?.disconnected
        ) {
            return;
        }

        this._socketService.emit('leave_doc', { docId });
        this._logger.log(`Left doc: ${docId}`);
    }

    async sendChangeset(changeset: IChangeset): Promise<void> {
        const composedMutations = this._transformService.composeList(
            changeset.mutations
        );
        const mutationsWithOpId: IMutationWithOpId[] = composedMutations.map((mutation) => ({
            ...mutation,
            opId: generateRandomId(32),
        }));
        const changesetWithOpId: IChangeset = {
            ...changeset,
            mutations: composedMutations,
        };
        await this._pendingMutationService.add(
            changeset.unitId,
            mutationsWithOpId,
            changeset.baseRev
        );
        const pendingAfterAdd = this._pendingMutationService.get(changeset.unitId) ?? [];
        // 更新保存状态：有未保存的操作
        this._updateSavedStatus(changeset.unitId);
        this._logger.debug(
            `sendChangeset: unit=${changeset.unitId}, baseRev=${changeset.baseRev}, ` +
                `added=${composedMutations.length}, pendingLen=${pendingAfterAdd.length}, ` +
                `socketDisconnected=${Boolean(this._socketService.getSocket()?.disconnected)}`
        );
        if (
            !this._socketService.getSocket() ||
            this._socketService.getSocket()?.disconnected
        ) {
            this._logger.debug(
                `sendChangeset: skip emit (socket disconnected), unit=${changeset.unitId}`
            );
            return;
        }
        this._logger.log(`Sending changeset: ${JSON.stringify(changesetWithOpId)}`);

        // Extract unitId from mutations (unitId is used as docId)
        const unitId = changeset.unitId;
        if (!unitId) {
            this._logger.error('Cannot determine unitId from changeset');
            return Promise.reject(
                new Error('Cannot determine unitId from changeset')
            );
        }

        // Ensure joined to doc
        if (!this._joinedDocs.has(unitId)) {
            this._logger.warn(`Not joined to doc: ${unitId}`);
            await this.joinDoc(unitId);
        }

        // Initialize queue if not exists
        if (!this._pendingMutationService.has(unitId)) {
            this._pendingMutationService.update(unitId, [], changeset.baseRev);
        }

        const queue = this._pendingMutationService.get(unitId) ?? [];
        this._logger.debug(
            `sendChangeset: queueLen=${queue.length}, batchLimit=${this._batchSizeLimit}, unit=${unitId}`
        );

        // Check if reached batch size limit
        if (queue.length >= this._batchSizeLimit) {
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
            this._logger.debug(`debounce: cleared timer for unit=${unitId}`);
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
            this._logger.debug(`debounce: trigger flush for unit=${unitId}`);
            await this._flushChangeset(unitId);
        }, this._debounceDelay);

        this._debounceTimers.set(unitId, timer);
        this._logger.debug(
            `debounce: set timer for unit=${unitId}, delay=${this._debounceDelay}`
        );
    }

    /**
     * Flush pending mutations for a specific unitId
     */
    private async _flushChangeset(
        unitId: string,
        mutations?: IMutationWithOpId[]
    ): Promise<void> {
        // Clear debounce timer
        this._clearDebounceTimer(unitId);

        if (this._inFlightFlushes.has(unitId)) {
            this._logger.debug(`flush: skip (in-flight), unit=${unitId}`);
            return;
        }

        const pendingMutations =
            mutations ?? this._pendingMutationService.get(unitId);

        // If no pending mutations, return
        if (!pendingMutations || pendingMutations.length === 0) {
            this._logger.debug(
                `flush: skip (no pending mutations), unit=${unitId}`
            );
            return;
        }

        const currentBaseRev = this.getDocRev(unitId);

        if (mutations) {
            await this._pendingMutationService.update(unitId, pendingMutations, currentBaseRev);
        }
        const opIds = pendingMutations.map((mutation) => mutation.opId);

        this._logger.log(
            `flush: unit=${unitId}, baseRev=${currentBaseRev}, sendLen=${pendingMutations.length}`
        );
        this._logger.debug(
            `flush: socketDisconnected=${Boolean(this._socketService.getSocket()?.disconnected)}, unit=${unitId}`
        );
        this._logger.debug(`flush: opIds=${opIds.join(',')}, unit=${unitId}`);

        // Build request
        const request: IChangesetRequest = {
            baseRev: currentBaseRev,
            mutations: pendingMutations,
            docId: unitId,
            clientId: this._socketService.getSocket()?.id,
        };

        this._logger.debug(
            `flush: request unit=${unitId}, mutations=${pendingMutations.length}`
        );

        // Send to server
        return new Promise<void>((resolve, reject) => {
            this._inFlightFlushes.add(unitId);
            this._socketService.emit(
                'changeset',
                request,
                async (ack: IChangesetAck) => {
                    this._logger.debug(
                        `flush: ack status=${ack.status}, serverRev=${ack.serverRev ?? 'null'}, ` +
                            `unit=${unitId}`
                    );
                    if (ack.status === 'ok' && ack.serverRev !== undefined) {
                        this.updateDocRev(unitId, ack.serverRev);

                        // Server returns transformed mutations - this is the authoritative result
                        if (ack.mutations && ack.mutations.length > 0) {
                            this._logger.log(
                                `Ack received with ${ack.mutations.length} transformed mutations (server_rev: ${ack.serverRev})`
                            );

                            // Optional: Verify consistency between client and server transform
                            // If there's a discrepancy, we should trust the server's result
                            const currentPending =
                                await this._pendingMutationService.get(unitId);
                            if (currentPending && currentPending.length > 0) {
                                this._logger.warn(
                                    `Client still has ${currentPending.length} pending mutations after Ack. ` +
                                        'This may indicate concurrent operations during Ack processing.'
                                );
                            }

                            // The server's transformed mutations are now the source of truth
                            // Clear pending since server has processed our operations
                            await this._pendingMutationService.removeByOpIds(unitId, opIds);
                            this._logger.log(`flush ack removed: unit=${unitId}, opIds=${opIds.length}`);
                            // 更新保存状态
                            this._updateSavedStatus(unitId);

                            // Note: We don't re-apply ack.mutations here because:
                            // 1. The operations were already applied locally when sent
                            // 2. Any concurrent operations have been handled via changeset_pushed broadcasts
                            // 3. The server's mutations serve as a verification/audit trail
                        } else {
                            this._logger.log(
                                `Changeset applied, new version: ${ack.serverRev}, mutations count: ${pendingMutations.length}`
                            );
                            await this._pendingMutationService.removeByOpIds(unitId, opIds);
                            this._logger.log(`flush ack removed: unit=${unitId}, opIds=${opIds.length}`);
                            // 更新保存状态
                            this._updateSavedStatus(unitId);
                        }

                        this._inFlightFlushes.delete(unitId);
                        await this._flushChangeset(unitId);
                        resolve();
                    } else {
                        // TODO: Handle version mismatch - may need to fetch_ops and resync
                        this._inFlightFlushes.delete(unitId);
                        await this._flushChangeset(unitId);
                        reject(new Error(`Changeset failed: ${ack.message}`));
                    }
                }
            );
        });
    }

    /**
     * Fetch operations from server since startRev
     */
    async fetchOps(docId: string, startRev: number): Promise<IOperationInfo[]> {
        if (
            !this._socketService.getSocket() ||
            this._socketService.getSocket()?.disconnected
        ) {
            this._logger.error('Socket not connected');
            return Promise.reject(new Error('Socket not connected'));
        }

        const request: IFetchOpsRequest = { docId, startRev };
        return new Promise<IOperationInfo[]>((resolve, reject) => {
            this._socketService.emit('fetch_ops', request, (ack: IFetchOpsAck) => {
                if (ack.status === 'ok' && ack.operations) {
                    this._logger.log(
                        `Fetched ${ack.operations.length} operations for doc ${docId} since rev ${startRev}`
                    );
                    resolve(ack.operations);
                } else {
                    this._logger.error(`Failed to fetch ops: ${ack.message}`);
                    reject(new Error(`Failed to fetch ops: ${ack.message}`));
                }
            });
        });
    }

    /**
     * Flush pending mutations immediately
     * @param unitId If provided, only flush this unitId's queue. Otherwise flush all.
     */
    async flush(unitId: string): Promise<void> {
        // Flush specific unitId
        await this._flushChangeset(unitId);
    }

    override dispose(): void {
        super.dispose();

        this._docJoined$.complete();
        this._docLeft$.complete();
        this._ready$.complete();
        this._connectStatus$.complete();

        // Complete and clear all saved status BehaviorSubjects
        for (const unitId in this._savedSubjects) {
            this._savedSubjects[unitId].complete();
        }
        this._savedSubjects = {};

        // Clear all debounce timers
        for (const unitId of this._debounceTimers.keys()) {
            this._clearDebounceTimer(unitId);
        }

        // Leave all joined docs
        for (const docId of this._joinedDocs) {
            this.leaveDoc(docId);
        }
    }
}
