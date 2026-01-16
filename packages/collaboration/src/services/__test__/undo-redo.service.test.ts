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

import type { ICommandService, IContextService, IMutationInfo, IUniverInstanceService } from '@univerjs/core';
import type { ITransformService } from '../transform.service';
import { CommandType, UniverInstanceType } from '@univerjs/core';
import { describe, expect, it } from 'vitest';
import { TransformService } from '../transform.service';
import { CollaborationUndoRedoService } from '../undo-redo.service';
import {
    createInsertRowMutation,
    createRange,
    createSetRangeValuesMutation,
    MockCommandService,
    MockContextService,
    MockLogService,
    MockUnit,
    MockUniverInstanceService,
    TEST_UNIT_ID,
} from './test-utils';

describe('CollaborationUndoRedoService', () => {
    it('transforms undo/redo stacks on remote mutation', () => {
        const instanceService = new MockUniverInstanceService();
        const commandService = new MockCommandService();
        const contextService = new MockContextService();
        const transformService = new TransformService();
        const logService = new MockLogService();

        const unit = new MockUnit(TEST_UNIT_ID, UniverInstanceType.UNIVER_SHEET, 0);
        instanceService.addUnit(unit);
        instanceService.focusUnit(TEST_UNIT_ID);

        const service = new CollaborationUndoRedoService(
            instanceService as unknown as IUniverInstanceService,
            commandService as unknown as ICommandService,
            contextService as unknown as IContextService,
            transformService,
            logService
        );

        const localUndo = createSetRangeValuesMutation({
            params: {
                unitId: TEST_UNIT_ID,
                subUnitId: 'sheet-1',
                cellValue: {
                    0: { 0: { v: '1' } },
                },
            },
        });
        const localRedo = createSetRangeValuesMutation({
            params: {
                unitId: TEST_UNIT_ID,
                subUnitId: 'sheet-1',
                cellValue: {
                    0: { 0: { v: '1' } },
                },
            },
        });

        service.pushUndoRedo({
            unitID: TEST_UNIT_ID,
            undoMutations: [localUndo],
            redoMutations: [localRedo],
        });

        const remoteMutation = createInsertRowMutation({
            params: {
                unitId: TEST_UNIT_ID,
                subUnitId: 'sheet-1',
                range: createRange({ startRow: 0, endRow: 0 }),
            },
        });

        commandService.emitMutationExecutedForCollab(
            {
                id: remoteMutation.id,
                type: CommandType.MUTATION,
                params: remoteMutation.params,
            },
            { fromCollab: true }
        );

        const top = service.pitchTopUndoElement();
        const nextParams = top?.undoMutations[0]?.params as { cellValue?: Record<string, Record<string, { v: string }>> } | undefined;

        expect(nextParams?.cellValue?.['1']?.['0']).toBeDefined();
        expect(nextParams?.cellValue?.['0']).toBeUndefined();

        service.dispose();
        transformService.dispose();
    });

    it('clears stacks when transform fails', () => {
        const instanceService = new MockUniverInstanceService();
        const commandService = new MockCommandService();
        const contextService = new MockContextService();
        const logService = new MockLogService();

        const failingTransform: ITransformService = {
            compose: () => [],
            transform: (m1: IMutationInfo, m2: IMutationInfo) => ({
                m1Prime: m1,
                m2Prime: m2,
                error: 'boom',
            }),
            transformList: () => ({
                m1Primes: [],
                m2Primes: [],
                error: 'boom',
            }),
        };

        const unit = new MockUnit(TEST_UNIT_ID, UniverInstanceType.UNIVER_SHEET, 0);
        instanceService.addUnit(unit);
        instanceService.focusUnit(TEST_UNIT_ID);

        const service = new CollaborationUndoRedoService(
            instanceService as unknown as IUniverInstanceService,
            commandService as unknown as ICommandService,
            contextService as unknown as IContextService,
            failingTransform,
            logService
        );

        service.pushUndoRedo({
            unitID: TEST_UNIT_ID,
            undoMutations: [createSetRangeValuesMutation()],
            redoMutations: [createSetRangeValuesMutation()],
        });

        const remoteMutation = createInsertRowMutation();
        commandService.emitMutationExecutedForCollab(
            {
                id: remoteMutation.id,
                type: CommandType.MUTATION,
                params: remoteMutation.params,
            },
            { fromCollab: true }
        );

        expect(service.pitchTopUndoElement()).toBeNull();
        service.dispose();
    });
});
