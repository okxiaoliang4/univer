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

import type { IConfigService, IMutationInfo, IUniverInstanceService } from '@univerjs/core';
import type { ICollaborationService } from '../../services/collaboration.service';
import type { IOperationInfo } from '../../services/socket.service';
import { UniverInstanceType } from '@univerjs/core';
import { describe, expect, it, vi } from 'vitest';
import {
    createInsertRowMutation,
    createRange,
    createSetRangeValuesMutation,
    MockCommandService,
    MockConfigService,
    MockLogService,
    MockSocketService,
    MockUnit,
    MockUniverInstanceService,
    TEST_UNIT_ID,
} from '../../services/__test__/test-utils';
import { TransformService } from '../../services/transform.service';
import { CollaborationController } from '../collaboration.controller';
import { COLLABORATION_PLUGIN_CONFIG_KEY } from '../config.schema';

const flushPromises = () => new Promise((resolve) => setTimeout(resolve, 0));

class MockCollaborationService implements ICollaborationService {
    pendingMutations: IMutationInfo[] = [];
    currentVersion?: number;
    nextFetchOps: IOperationInfo[] = [];
    nextSyncResult: { missedOps: IOperationInfo[]; pendingMutations: IMutationInfo[]; serverVersion: number } = {
        missedOps: [],
        pendingMutations: [],
        serverVersion: 0,
    };

    sendChangeset = vi.fn();
    joinDoc = vi.fn(async () => {});
    leaveDoc = vi.fn();
    flush = vi.fn(async () => {});
    fetchOps = vi.fn(async () => this.nextFetchOps);
    syncOnReconnect = vi.fn(async () => this.nextSyncResult);
    getPendingMutations = vi.fn(() => this.pendingMutations);
    setTransformedPendingMutations = vi.fn((unitId: string, mutations: IMutationInfo[], _baseRev: number) => {
        this.pendingMutations = mutations;
    });

    getCurrentVersion = vi.fn(() => this.currentVersion);
    setCurrentVersion = vi.fn((_unitId: string, version: number) => {
        this.currentVersion = version;
    });
}

describe('CollaborationController', () => {
    it('creates socket and disconnects on dispose', () => {
        const socketService = new MockSocketService();
        const configService = new MockConfigService({
            [COLLABORATION_PLUGIN_CONFIG_KEY]: { wsUrl: 'ws://test', userId: 'user-1' },
        });
        const commandService = new MockCommandService();
        const instanceService = new MockUniverInstanceService();
        const collaborationService = new MockCollaborationService();
        const transformService = new TransformService();
        const logService = new MockLogService();

        const controller = new CollaborationController(
            socketService,
            configService as unknown as IConfigService,
            commandService,
            instanceService as unknown as IUniverInstanceService,
            collaborationService,
            transformService,
            logService
        );

        expect(socketService.createdUrl).toBe('ws://test');
        controller.dispose();

        expect(socketService.getSocket()?.disconnected).toBe(true);
        transformService.dispose();
    });

    it('transforms pending mutations on changeset push without version gap', async () => {
        const socketService = new MockSocketService();
        const configService = new MockConfigService({
            [COLLABORATION_PLUGIN_CONFIG_KEY]: { wsUrl: 'ws://test', userId: 'user-1' },
        });
        const commandService = new MockCommandService();
        const instanceService = new MockUniverInstanceService();
        const collaborationService = new MockCollaborationService();
        const transformService = new TransformService();
        const logService = new MockLogService();

        const unit = new MockUnit(TEST_UNIT_ID, UniverInstanceType.UNIVER_SHEET, 1);
        instanceService.addUnit(unit);

        const pending = [createSetRangeValuesMutation()];
        const incoming = [createInsertRowMutation()];
        collaborationService.pendingMutations = pending;

        const controller = new CollaborationController(
            socketService,
            configService as unknown as IConfigService,
            commandService,
            instanceService as unknown as IUniverInstanceService,
            collaborationService,
            transformService,
            logService
        );

        const expected = transformService.transformList(pending, incoming);
        socketService.emitChangesetPushed({
            docId: TEST_UNIT_ID,
            serverRev: 2,
            userId: 'user-2',
            mutations: incoming,
        });

        await flushPromises();

        expect(commandService.executed).toHaveLength(expected.m2Primes.length);
        expect(collaborationService.setTransformedPendingMutations).toHaveBeenCalledWith(
            TEST_UNIT_ID,
            expected.m1Primes,
            2
        );
        expect(collaborationService.setCurrentVersion).toHaveBeenCalledWith(TEST_UNIT_ID, 2);
        expect(unit.getRev()).toBe(2);

        controller.dispose();
        transformService.dispose();
    });

    it('fetches missed ops on version gap when no pending mutations', async () => {
        const socketService = new MockSocketService();
        const configService = new MockConfigService({
            [COLLABORATION_PLUGIN_CONFIG_KEY]: { wsUrl: 'ws://test', userId: 'user-1' },
        });
        const commandService = new MockCommandService();
        const instanceService = new MockUniverInstanceService();
        const collaborationService = new MockCollaborationService();
        const transformService = new TransformService();
        const logService = new MockLogService();

        const unit = new MockUnit(TEST_UNIT_ID, UniverInstanceType.UNIVER_SHEET, 1);
        instanceService.addUnit(unit);

        collaborationService.pendingMutations = [];
        collaborationService.nextFetchOps = [
            {
                rev: 2,
                userId: 'user-2',
                mutations: [createSetRangeValuesMutation()],
            },
            {
                rev: 3,
                userId: 'user-3',
                mutations: [createInsertRowMutation()],
            },
        ];

        const controller = new CollaborationController(
            socketService,
            configService as unknown as IConfigService,
            commandService,
            instanceService as unknown as IUniverInstanceService,
            collaborationService,
            transformService,
            logService
        );

        socketService.emitChangesetPushed({
            docId: TEST_UNIT_ID,
            serverRev: 4,
            userId: 'user-4',
            mutations: [createInsertRowMutation()],
        });

        await flushPromises();

        expect(collaborationService.fetchOps).toHaveBeenCalledWith(TEST_UNIT_ID, 2);
        expect(commandService.executed).toHaveLength(3);
        expect(unit.getRev()).toBe(4);

        controller.dispose();
        transformService.dispose();
    });

    it('syncs pending and missed ops on reconnect', async () => {
        const socketService = new MockSocketService();
        const configService = new MockConfigService({
            [COLLABORATION_PLUGIN_CONFIG_KEY]: { wsUrl: 'ws://test', userId: 'user-1' },
        });
        const commandService = new MockCommandService();
        const instanceService = new MockUniverInstanceService();
        const collaborationService = new MockCollaborationService();
        const transformService = new TransformService();
        const logService = new MockLogService();

        const unit = new MockUnit(TEST_UNIT_ID, UniverInstanceType.UNIVER_SHEET, 1);
        instanceService.addUnit(unit);

        const pending = [createSetRangeValuesMutation()];
        const missedOps: IOperationInfo[] = [
            {
                rev: 5,
                userId: 'user-2',
                mutations: [
                    createInsertRowMutation({
                        params: {
                            unitId: TEST_UNIT_ID,
                            subUnitId: 'sheet-1',
                            range: createRange({ startRow: 0, endRow: 0 }),
                        },
                    }),
                ],
            },
        ];

        collaborationService.nextSyncResult = {
            missedOps,
            pendingMutations: pending,
            serverVersion: 6,
        };

        const controller = new CollaborationController(
            socketService,
            configService as unknown as IConfigService,
            commandService,
            instanceService as unknown as IUniverInstanceService,
            collaborationService,
            transformService,
            logService
        );

        const expected = transformService.transformList(pending, missedOps.flatMap((op) => op.mutations));
        socketService.emitConnected();
        await flushPromises();

        expect(commandService.executed).toHaveLength(expected.m2Primes.length);
        expect(collaborationService.setTransformedPendingMutations).toHaveBeenCalledWith(TEST_UNIT_ID, expected.m1Primes, 6);
        expect(collaborationService.flush).toHaveBeenCalledWith(TEST_UNIT_ID);
        expect(collaborationService.setCurrentVersion).toHaveBeenCalledWith(TEST_UNIT_ID, 5);
        expect(unit.getRev()).toBe(5);

        controller.dispose();
        transformService.dispose();
    });
});
