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

import type { IChangeset, IOperationInfo } from '../socket.service';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { CollaborationService } from '../collaboration.service';
import {
    createInsertRowMutation,
    createSetRangeValuesMutation,
    MockLogService,
    MockOfflineStorageService,
    MockSocketService,
    TEST_UNIT_ID,
} from './test-utils';

describe('CollaborationService', () => {
    let socketService: MockSocketService;
    let offlineStorage: MockOfflineStorageService;
    let logService: MockLogService;
    let service: CollaborationService;

    beforeEach(() => {
        socketService = new MockSocketService();
        socketService.createSocket('ws://test');
        socketService.setSocketState(true);
        offlineStorage = new MockOfflineStorageService();
        logService = new MockLogService();
        service = new CollaborationService(socketService, logService, offlineStorage);
    });

    afterEach(() => {
        service.dispose();
    });

    it('stores pending mutations when socket is disconnected', async () => {
        socketService.setSocketState(false);
        const changeset: IChangeset = {
            unitId: TEST_UNIT_ID,
            baseRev: 0,
            userId: 'user-1',
            mutations: [createSetRangeValuesMutation()],
        };

        await expect(service.sendChangeset(changeset)).rejects.toThrow('Socket not connected');
        const stored = await offlineStorage.loadPendingMutations(TEST_UNIT_ID);
        expect(stored?.mutations).toHaveLength(1);
    });

    it('flushes changeset and updates version on ack ok', async () => {
        socketService.nextJoinAck = { status: 'ok', version: 1 };
        socketService.nextChangesetAck = { status: 'ok', serverRev: 2 };
        const changeset: IChangeset = {
            unitId: TEST_UNIT_ID,
            baseRev: 1,
            userId: 'user-1',
            mutations: [createSetRangeValuesMutation()],
        };

        await service.sendChangeset(changeset);
        await service.flush(TEST_UNIT_ID);

        expect(service.getDocRev(TEST_UNIT_ID)).toBe(2);
        expect(offlineStorage.clearCalls).toContain(TEST_UNIT_ID);
    });

    it('requeues mutations and saves offline on ack error', async () => {
        socketService.nextJoinAck = { status: 'ok', version: 1 };
        socketService.nextChangesetAck = { status: 'error', message: 'fail' };
        const changeset: IChangeset = {
            unitId: TEST_UNIT_ID,
            baseRev: 1,
            userId: 'user-1',
            mutations: [createSetRangeValuesMutation()],
        };

        await service.sendChangeset(changeset);
        await expect(service.flush(TEST_UNIT_ID)).rejects.toThrow('Changeset failed');

        expect(service.getPendingMutations(TEST_UNIT_ID)).toHaveLength(1);
        const stored = await offlineStorage.loadPendingMutations(TEST_UNIT_ID);
        expect(stored?.mutations).toHaveLength(1);
    });

    it('syncOnReconnect merges offline pending and fetches missed ops', async () => {
        socketService.nextJoinAck = { status: 'ok', version: 5 };
        socketService.nextFetchOpsAck = {
            status: 'ok',
            operations: [
                {
                    rev: 3,
                    userId: 'user-2',
                    mutations: [createInsertRowMutation()],
                } as IOperationInfo,
            ],
        };

        await offlineStorage.savePendingMutations(TEST_UNIT_ID, [createSetRangeValuesMutation()], 2, 'user-1');
        service.updateDocRev(TEST_UNIT_ID, 2);

        const result = await service.syncOnReconnect(TEST_UNIT_ID);
        expect(result.pendingMutations).toHaveLength(1);
        expect(result.missedOps).toHaveLength(1);

        expect(service.getPendingBaseRev(TEST_UNIT_ID)).toBe(2);

        const fetchCall = socketService.emitted.find((entry) => entry.event === 'fetch_ops');
        expect(fetchCall?.args[0]).toEqual({ docId: TEST_UNIT_ID, startRev: 2 });
    });
});
