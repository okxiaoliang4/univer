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
import type { ICollaborationConfig } from './config.schema';
import { Disposable, ICommandService, IConfigService, ILogService, isInternalEditorID, IUniverInstanceService, sequenceExecute, toDisposable, UniverInstanceType } from '@univerjs/core';
import { ICollaborationService } from '../services/collaboration.service';
import { ISocketService } from '../services/socket.service';
import { ITransformService } from '../services/transform.service';
import { COLLABORATION_PLUGIN_CONFIG_KEY } from './config.schema';

export class CollaborationController extends Disposable {
    private _localMutationSignatures: Map<string, Map<string, number>> = new Map();

    constructor(
        @ISocketService private readonly _socketService: ISocketService,
        @IConfigService private readonly _configService: IConfigService,
        @ICommandService private readonly _commandService: ICommandService,
        @IUniverInstanceService private readonly _univerInstanceService: IUniverInstanceService,
        @ICollaborationService private readonly _collaborationService: ICollaborationService,
        @ITransformService private readonly _transformService: ITransformService,
        @ILogService private readonly _logger: ILogService
    ) {
        super();
        this._init();
        this._initListener();
    }

    private _init(): void {
        this._initSocket();
    }

    private _initSocket(): void {
        const config = this._configService.getConfig<ICollaborationConfig>(COLLABORATION_PLUGIN_CONFIG_KEY)!;
        const socket = this._socketService.createSocket(config.wsUrl);
        if (!socket) {
            throw new Error('Failed to create socket');
        }
        this.disposeWithMe(toDisposable(() => socket.disconnect()));
    }

    private _initListener(): void {
        this._initInstanceListener();
        this._initCommandListener();
        this._initChangesetPushedListener();
        this._initReconnectionSync();
    }

    private _initInstanceListener(): void {
        this.disposeWithMe(this._univerInstanceService.unitAdded$.subscribe(async (unit) => {
            const unitId = unit.getUnitId();
            if (isInternalEditorID(unitId)) return;

            try {
                // Join doc and get server version
                await this._collaborationService.joinDoc(unitId);

                // After joining, check if we need to fetch ops
                // This handles the case where page is refreshed and snapshot is loaded but ops are missing
                await this._syncVersionAfterJoin(unitId, unit);
            } catch (error) {
                this._logger.error(`Failed to join doc ${unitId}:`, error);
            }
        }));

        this.disposeWithMe(this._univerInstanceService.unitDisposed$.subscribe((workbook) => {
            const unitId = workbook.getUnitId();
            if (isInternalEditorID(unitId)) return;
            this._collaborationService.leaveDoc(unitId);
        }));
    }

    /**
     * Sync version after joining a doc - fetch missed ops if local version is behind
     */
    private async _syncVersionAfterJoin(unitId: string, unit: UnitModel): Promise<void> {
        const localRev = unit.getRev();

        // Fetch ops starting from local version to get any updates since snapshot was created
        // This handles the case where:
        // 1. Page was refreshed and loaded from snapshot
        // 2. Snapshot might be outdated compared to current server version
        try {
            const missedOps = await this._collaborationService.fetchOps(unitId, localRev);

            if (missedOps.length > 0) {
                this._logger.log(`Found ${missedOps.length} missed operations for doc ${unitId} since rev ${localRev}`);

                // Apply missed mutations
                for (const op of missedOps) {
                    await sequenceExecute(op.mutations, this._commandService, { fromCollab: true });
                    unit.setRev(op.rev);
                }

                this._logger.log(`Applied missed operations, unit rev now: ${unit.getRev()}`);
                this._collaborationService.setCurrentVersion(unitId, unit.getRev());
            }
        } catch (error) {
            this._logger.error(`Failed to fetch ops for ${unitId} after join:`, error);
        }
    }

    private _initCommandListener(): void {
        const config = this._configService.getConfig<ICollaborationConfig>(COLLABORATION_PLUGIN_CONFIG_KEY)!;
        this.disposeWithMe(this._commandService.onMutationExecutedForCollab((command, options) => {
            const commandParams = command.params as { unitId?: string } | undefined;
            const observedUnitId = commandParams?.unitId;
            // #region agent log
            fetch('http://127.0.0.1:7242/ingest/602f28cd-f78b-4388-a3f1-b1ee0e32b82f', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ sessionId: 'debug-session', runId: 'pre-fix', hypothesisId: 'H6', location: 'collaboration.controller.ts:commandListener', message: 'mutation observed by collab listener', data: { mutationId: command.id, mutationType: command.type, fromCollab: Boolean(options?.fromCollab), unitId: observedUnitId, hasUnitId: Boolean(observedUnitId) }, timestamp: Date.now() }) }).catch(() => {});
            // #endregion
            if (options?.fromCollab) return;
            const unitId = (command.params as { unitId: string })?.unitId;
            if (!unitId) return;
            const unit = this._univerInstanceService.getUnit(unitId);
            if (!unit || isInternalEditorID(unitId)) return;
            this._trackLocalMutation(unitId, command as IMutationInfo);
            const baseRev = this._collaborationService.getCurrentVersion(unitId) ?? unit.getRev();
            this._collaborationService.sendChangeset({
                unitId,
                baseRev,
                userId: config.userId,
                mutations: [command as IMutationInfo],
            });
        }));
    }

    private _initChangesetPushedListener(): void {
        const config = this._configService.getConfig<ICollaborationConfig>(COLLABORATION_PLUGIN_CONFIG_KEY)!;
        this.disposeWithMe(this._socketService.changesetPushed$.subscribe(async (changeset) => {
            const unitId = changeset.docId;
            const unit = this._univerInstanceService.getUnit(unitId);

            if (!unit || isInternalEditorID(unitId)) {
                this._logger.warn(`Unit not found or is internal: ${unitId}`);
                return;
            }

            const localRev = unit.getRev();
            const expectedRev = localRev + 1;
            const serverRev = changeset.serverRev;
            const localUserId = config.userId;
            const isSelfChangeset = changeset.userId === localUserId;
            const filteredChangesetMutations = this._filterLocalEchoMutations(
                unitId,
                changeset.mutations,
                localUserId,
                changeset.userId,
                'changeset_pushed'
            );
            const incomingIdCounts = filteredChangesetMutations.reduce<Record<string, number>>((acc, mutation) => {
                acc[mutation.id] = (acc[mutation.id] ?? 0) + 1;
                return acc;
            }, {});
            // #region agent log
            fetch('http://127.0.0.1:7242/ingest/602f28cd-f78b-4388-a3f1-b1ee0e32b82f', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ sessionId: 'debug-session', runId: 'pre-fix', hypothesisId: 'H5', location: 'collaboration.controller.ts:changesetPushed:incoming', message: 'changeset_pushed received summary', data: { unitId, serverRev, localRev, pendingCount: this._collaborationService.getPendingMutations(unitId).length, incomingCount: changeset.mutations.length, incomingIdCounts, localUserId, changesetUserId: changeset.userId, isSelfChangeset }, timestamp: Date.now() }) }).catch(() => {});
            // #endregion

            // Check for version gap - need to fetch missed operations
            if (serverRev > expectedRev) {
                this._logger.warn(`Version gap detected: local=${localRev}, expected=${expectedRev}, server=${serverRev}`);

                try {
                    const missedOps = await this._collaborationService.fetchOps(unitId, expectedRev);
                    const missedLocalOpCount = missedOps.filter((op) => op.userId === localUserId).length;
                    const missedLocalMutationCount = missedOps.reduce((count, op) => (op.userId === localUserId ? count + op.mutations.length : count), 0);

                    // Combine missed ops with incoming changeset
                    const allMissedMutations: IMutationInfo[] = [];
                    for (const op of missedOps) {
                        const filteredMutations = this._filterLocalEchoMutations(
                            unitId,
                            op.mutations,
                            localUserId,
                            op.userId,
                            'changeset_pushed_missed'
                        );
                        allMissedMutations.push(...filteredMutations);
                    }
                    allMissedMutations.push(...filteredChangesetMutations);

                    // Get pending mutations
                    const pendingMutations = this._collaborationService.getPendingMutations(unitId);
                    const pendingIdCounts = pendingMutations.reduce<Record<string, number>>((acc, mutation) => {
                        acc[mutation.id] = (acc[mutation.id] ?? 0) + 1;
                        return acc;
                    }, {});
                    const missedIdCounts = allMissedMutations.reduce<Record<string, number>>((acc, mutation) => {
                        acc[mutation.id] = (acc[mutation.id] ?? 0) + 1;
                        return acc;
                    }, {});
                    // #region agent log
                    fetch('http://127.0.0.1:7242/ingest/602f28cd-f78b-4388-a3f1-b1ee0e32b82f', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ sessionId: 'debug-session', runId: 'pre-fix', hypothesisId: 'H4', location: 'collaboration.controller.ts:changesetPushed:missedSummary', message: 'changeset_pushed missed ops + pending summary', data: { unitId, serverRev, localRev, pendingCount: pendingMutations.length, missedCount: allMissedMutations.length, pendingIdCounts, missedIdCounts, localUserId, missedLocalOpCount, missedLocalMutationCount }, timestamp: Date.now() }) }).catch(() => {});
                    // #endregion

                    if (pendingMutations.length > 0) {
                        // OT transform: apply transformed server mutations locally
                        const result = this._transformService.transformList(pendingMutations, allMissedMutations);
                        if (result.error) {
                            this._logger.error(`Transform error: ${result.error}`);
                            await sequenceExecute(allMissedMutations, this._commandService, { fromCollab: true });
                        } else {
                            await sequenceExecute(result.m2Primes, this._commandService, { fromCollab: true });
                            const pendingBaseRev = this._collaborationService.getPendingBaseRev(unitId);
                            this._collaborationService.setTransformedPendingMutations(unitId, result.m1Primes, pendingBaseRev ?? serverRev);
                        }
                    } else {
                        // No pending mutations, apply directly
                        for (const op of missedOps) {
                            await sequenceExecute(op.mutations, this._commandService, { fromCollab: true });
                        }
                        await sequenceExecute(changeset.mutations, this._commandService, { fromCollab: true });
                    }
                } catch (error) {
                    this._logger.error('Failed to fetch missed operations:', error);
                    // Fall back to applying incoming changeset directly
                    if (filteredChangesetMutations.length > 0) {
                        await sequenceExecute(filteredChangesetMutations, this._commandService, { fromCollab: true });
                    }
                }
            } else {
                // Normal case: no version gap
                const pendingMutations = this._collaborationService.getPendingMutations(unitId);

                this._logger.log(`Received changeset: serverRev=${serverRev}, localRev=${localRev}, pendingMutations=${pendingMutations.length}, incomingMutations=${filteredChangesetMutations.length}`);

                if (pendingMutations.length > 0) {
                    // OT transform: apply transformed server mutations locally
                    const result = this._transformService.transformList(pendingMutations, filteredChangesetMutations);
                    if (result.error) {
                        this._logger.error(`Transform error: ${result.error}`);
                        if (filteredChangesetMutations.length > 0) {
                            await sequenceExecute(filteredChangesetMutations, this._commandService, { fromCollab: true });
                        }
                    } else {
                        this._logger.log(`OT result: m1Primes=${result.m1Primes.length}, m2Primes=${result.m2Primes.length}`);

                        // Apply transformed server mutations
                        if (result.m2Primes.length > 0) {
                            await sequenceExecute(result.m2Primes, this._commandService, { fromCollab: true });
                        }

                        // Always update pending mutations with transformed versions
                        // Even if m1Primes is empty, we need to clear the old pending mutations
                        const pendingBaseRev = this._collaborationService.getPendingBaseRev(unitId);
                        this._collaborationService.setTransformedPendingMutations(unitId, result.m1Primes, pendingBaseRev ?? serverRev);
                    }
                } else {
                    // No pending mutations, apply directly
                    this._logger.log(`No pending mutations, applying ${filteredChangesetMutations.length} mutations directly`);
                    for (const mutation of filteredChangesetMutations) {
                        this._logger.log(`Executing mutation: id=${mutation.id}, params=${JSON.stringify(mutation.params)?.substring(0, 200)}`);
                    }
                    if (filteredChangesetMutations.length > 0) {
                        const result = await sequenceExecute(filteredChangesetMutations, this._commandService, { fromCollab: true });
                        this._logger.log(`sequenceExecute result: ${result}`);
                    }
                }
            }

            unit.setRev(serverRev);
            this._collaborationService.setCurrentVersion(unitId, serverRev);
        }));
    }

    private _initReconnectionSync(): void {
        const config = this._configService.getConfig<ICollaborationConfig>(COLLABORATION_PLUGIN_CONFIG_KEY)!;
        this.disposeWithMe(this._socketService.connected$.subscribe(async () => {
            this._logger.log('Socket reconnected, syncing all joined documents');

            // Get all units that are not internal (sheets and docs)
            const sheetUnits = this._univerInstanceService.getAllUnitsForType<UnitModel>(UniverInstanceType.UNIVER_SHEET);
            const docUnits = this._univerInstanceService.getAllUnitsForType<UnitModel>(UniverInstanceType.UNIVER_DOC);
            const units = [...sheetUnits, ...docUnits];

            for (const unit of units) {
                const unitId = unit.getUnitId();
                if (isInternalEditorID(unitId)) {
                    continue;
                }

                try {
                    const { missedOps, pendingMutations, serverVersion } = await this._collaborationService.syncOnReconnect(unitId);
                    // #region agent log
                    fetch('http://127.0.0.1:7242/ingest/602f28cd-f78b-4388-a3f1-b1ee0e32b82f', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ sessionId: 'debug-session', runId: 'pre-fix', hypothesisId: 'H3', location: 'collaboration.controller.ts:reconnect:syncResult', message: 'reconnect sync result snapshot', data: { unitId, unitRev: unit.getRev(), pendingCount: pendingMutations.length, missedOpsCount: missedOps.length, serverVersion }, timestamp: Date.now() }) }).catch(() => {});
                    // #endregion

                    // Flatten missed mutations
                    const localUserId = config.userId;
                    const missedMutations: IMutationInfo[] = [];
                    for (const op of missedOps) {
                        const filteredMutations = this._filterLocalEchoMutations(
                            unitId,
                            op.mutations,
                            localUserId,
                            op.userId,
                            'reconnect_missed'
                        );
                        missedMutations.push(...filteredMutations);
                    }
                    const missedLocalOpCount = missedOps.filter((op) => op.userId === localUserId).length;
                    const missedLocalMutationCount = missedOps.reduce((count, op) => (op.userId === localUserId ? count + op.mutations.length : count), 0);
                    const pendingIdCounts = pendingMutations.reduce<Record<string, number>>((acc, mutation) => {
                        acc[mutation.id] = (acc[mutation.id] ?? 0) + 1;
                        return acc;
                    }, {});
                    const missedIdCounts = missedMutations.reduce<Record<string, number>>((acc, mutation) => {
                        acc[mutation.id] = (acc[mutation.id] ?? 0) + 1;
                        return acc;
                    }, {});
                    // #region agent log
                    fetch('http://127.0.0.1:7242/ingest/602f28cd-f78b-4388-a3f1-b1ee0e32b82f', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ sessionId: 'debug-session', runId: 'pre-fix', hypothesisId: 'H4', location: 'collaboration.controller.ts:reconnect:missedSummary', message: 'reconnect missed ops + pending summary', data: { unitId, serverVersion, unitRev: unit.getRev(), pendingCount: pendingMutations.length, missedCount: missedMutations.length, pendingIdCounts, missedIdCounts, localUserId, missedLocalOpCount, missedLocalMutationCount }, timestamp: Date.now() }) }).catch(() => {});
                    // #endregion

                    // OT Sync Flow:
                    // - Local state = base + pendingMutations (already applied locally)
                    // - Server state = base + missedMutations
                    // - Need to apply m2Primes locally and send m1Primes to server
                    // - Result: both sides have base + pendingMutations + m2Primes = base + missedMutations + m1Primes

                    if (pendingMutations.length > 0 && missedMutations.length > 0) {
                        this._logger.log(`OT sync: transforming ${pendingMutations.length} local ops against ${missedMutations.length} server ops`);

                        const result = this._transformService.transformList(pendingMutations, missedMutations);
                        // #region agent log
                        fetch('http://127.0.0.1:7242/ingest/602f28cd-f78b-4388-a3f1-b1ee0e32b82f', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ sessionId: 'debug-session', runId: 'pre-fix', hypothesisId: 'H2', location: 'collaboration.controller.ts:reconnect:transform', message: 'reconnect transform result', data: { unitId, resultError: result.error ?? null, m1Count: result.m1Primes.length, m2Count: result.m2Primes.length, m1Ids: result.m1Primes.slice(0, 3).map((m) => m.id), m2Ids: result.m2Primes.slice(0, 3).map((m) => m.id) }, timestamp: Date.now() }) }).catch(() => {});
                        // #endregion
                        if (result.error) {
                            this._logger.error(`Transform error: ${result.error}`);
                            // Fall back to applying original missed mutations
                            for (const op of missedOps) {
                                await sequenceExecute(op.mutations, this._commandService, { fromCollab: true });
                            }
                        } else {
                            // Apply transformed server mutations (m2Primes) locally
                            // These are the server ops transformed to work on top of our local changes
                            this._logger.log(`Applying ${result.m2Primes.length} transformed server mutations`);
                            await sequenceExecute(result.m2Primes, this._commandService, { fromCollab: true });

                            // Update pending mutations with transformed versions (m1Primes)
                            // These will be sent to the server
                            const pendingBaseRev = this._collaborationService.getPendingBaseRev(unitId);
                            this._collaborationService.setTransformedPendingMutations(unitId, result.m1Primes, pendingBaseRev ?? serverVersion);
                        }
                    } else if (missedMutations.length > 0) {
                        // No pending mutations, just apply missed mutations directly
                        this._logger.log(`Applying ${missedOps.length} missed operations for doc ${unitId}`);
                        for (const op of missedOps) {
                            await sequenceExecute(op.mutations, this._commandService, { fromCollab: true });
                        }
                    }

                    // Update unit revision to server version
                    if (missedOps.length > 0) {
                        const lastOp = missedOps[missedOps.length - 1];
                        unit.setRev(lastOp.rev);
                        this._collaborationService.setCurrentVersion(unitId, lastOp.rev);
                    } else {
                        this._collaborationService.setCurrentVersion(unitId, serverVersion);
                    }
                    // #region agent log
                    fetch('http://127.0.0.1:7242/ingest/602f28cd-f78b-4388-a3f1-b1ee0e32b82f', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ sessionId: 'debug-session', runId: 'pre-fix', hypothesisId: 'H3', location: 'collaboration.controller.ts:reconnect:postApply', message: 'reconnect apply summary', data: { unitId, unitRev: unit.getRev(), currentVersion: this._collaborationService.getCurrentVersion(unitId), missedOpsCount: missedOps.length, pendingCount: this._collaborationService.getPendingMutations(unitId).length }, timestamp: Date.now() }) }).catch(() => {});
                    // #endregion

                    // Flush pending mutations (send transformed m1Primes to server)
                    if (pendingMutations.length > 0) {
                        this._logger.log(`Flushing ${pendingMutations.length} pending mutations for doc ${unitId}`);
                        try {
                            await this._collaborationService.flush(unitId);
                        } catch (error) {
                            this._logger.error(`Failed to flush pending mutations for ${unitId}:`, error);
                        }
                    }
                } catch (error) {
                    this._logger.error(`Failed to sync unit ${unitId} on reconnect:`, error);
                }
            }
        }));
    }

    private _trackLocalMutation(unitId: string, mutation: IMutationInfo): void {
        const signature = this._getMutationSignature(mutation);
        let unitSignatures = this._localMutationSignatures.get(unitId);
        if (!unitSignatures) {
            unitSignatures = new Map();
            this._localMutationSignatures.set(unitId, unitSignatures);
        }
        const nextCount = (unitSignatures.get(signature) ?? 0) + 1;
        unitSignatures.set(signature, nextCount);
        // #region agent log
        fetch('http://127.0.0.1:7242/ingest/602f28cd-f78b-4388-a3f1-b1ee0e32b82f', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ sessionId: 'debug-session', runId: 'pre-fix', hypothesisId: 'H9', location: 'collaboration.controller.ts:trackLocal', message: 'tracked local mutation signature', data: { unitId, mutationId: mutation.id, signatureCount: nextCount }, timestamp: Date.now() }) }).catch(() => {});
        // #endregion
    }

    private _filterLocalEchoMutations(
        unitId: string,
        mutations: IMutationInfo[],
        localUserId: string | undefined,
        opUserId: string | undefined,
        source: string
    ): IMutationInfo[] {
        if (!localUserId || !opUserId) {
            return mutations;
        }
        if (localUserId !== opUserId) {
            return mutations;
        }
        const removedCount = mutations.length;
        if (removedCount > 0) {
            // #region agent log
            fetch('http://127.0.0.1:7242/ingest/602f28cd-f78b-4388-a3f1-b1ee0e32b82f', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ sessionId: 'debug-session', runId: 'pre-fix', hypothesisId: 'H9', location: 'collaboration.controller.ts:filterLocalEcho', message: 'filtered local-echo mutations by userId', data: { unitId, source, removedCount, remainingCount: 0, localUserId, opUserId }, timestamp: Date.now() }) }).catch(() => {});
            // #endregion
        }
        return [];
    }

    private _getMutationSignature(mutation: IMutationInfo): string {
        try {
            return `${mutation.id}:${JSON.stringify(mutation.params)}`;
        } catch {
            return `${mutation.id}:<unstringifiable>`;
        }
    }
}
