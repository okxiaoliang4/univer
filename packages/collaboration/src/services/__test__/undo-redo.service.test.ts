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

import type { ICommandService, IContextService, IUniverInstanceService } from '@univerjs/core';
import type { IRPCChannelService } from '@univerjs/rpc';
import { CommandType, UniverInstanceType } from '@univerjs/core';
import { describe, expect, it, vi } from 'vitest';
import { UNDO_REDO_TRANSFORM_SERVICE_NAME } from '../../common/types';
import { CollaborationUndoRedoService } from '../undo-redo.service';
import {
    createInsertRowMutation,
    createSetRangeValuesMutation,
    MockCommandService,
    MockContextService,
    MockLogService,
    MockUnit,
    MockUniverInstanceService,
    TEST_UNIT_ID,
} from './test-utils';

/**
 * Mock RPC channel service for testing
 */
class MockRPCChannelService implements IRPCChannelService {
    private _channels = new Map<string, unknown>();

    requestChannel<T>(name: string): T {
        const channel = this._channels.get(name);
        if (!channel) {
            throw new Error(`Channel ${name} not found`);
        }
        return channel as T;
    }

    registerChannel(name: string, channel: unknown): void {
        this._channels.set(name, channel);
    }

    setChannel(name: string, channel: unknown): void {
        this._channels.set(name, channel);
    }
}

/**
 * Mock undo-redo transform service for testing
 */
function createMockUndoRedoTransformService() {
    return {
        transformUndoStack: vi.fn().mockResolvedValue({
            success: true,
            items: [],
        }),
        transformRedoStack: vi.fn().mockResolvedValue({
            success: true,
            items: [],
        }),
        transformBothStacks: vi.fn().mockResolvedValue({
            undoResult: { success: true, items: [] },
            redoResult: { success: true, items: [] },
        }),
    };
}

describe('CollaborationUndoRedoService', () => {
    it('marks stacks as dirty on remote mutation', () => {
        const instanceService = new MockUniverInstanceService();
        const commandService = new MockCommandService();
        const contextService = new MockContextService();
        const rpcService = new MockRPCChannelService();
        const logService = new MockLogService();

        const mockTransformService = createMockUndoRedoTransformService();
        rpcService.setChannel(UNDO_REDO_TRANSFORM_SERVICE_NAME, mockTransformService);

        const unit = new MockUnit(TEST_UNIT_ID, UniverInstanceType.UNIVER_SHEET, 0);
        instanceService.addUnit(unit);
        instanceService.focusUnit(TEST_UNIT_ID);

        const service = new CollaborationUndoRedoService(
            instanceService as unknown as IUniverInstanceService,
            commandService as unknown as ICommandService,
            contextService as unknown as IContextService,
            rpcService,
            logService
        );

        // Push an undo/redo item
        service.pushUndoRedo({
            unitID: TEST_UNIT_ID,
            undoMutations: [createSetRangeValuesMutation()],
            redoMutations: [createSetRangeValuesMutation()],
        });

        // Simulate remote mutation
        const remoteMutation = createInsertRowMutation();
        commandService.emitMutationExecutedForCollab(
            {
                id: remoteMutation.id,
                type: CommandType.MUTATION,
                params: remoteMutation.params,
            },
            { fromCollab: true }
        );

        // Verify stack still has items (lazy transform doesn't clear)
        const top = service.pitchTopUndoElement();
        expect(top).not.toBeNull();

        service.dispose();
    });

    it('ignores local mutations (fromCollab: false)', () => {
        const instanceService = new MockUniverInstanceService();
        const commandService = new MockCommandService();
        const contextService = new MockContextService();
        const rpcService = new MockRPCChannelService();
        const logService = new MockLogService();

        const mockTransformService = createMockUndoRedoTransformService();
        rpcService.setChannel(UNDO_REDO_TRANSFORM_SERVICE_NAME, mockTransformService);

        const unit = new MockUnit(TEST_UNIT_ID, UniverInstanceType.UNIVER_SHEET, 0);
        instanceService.addUnit(unit);
        instanceService.focusUnit(TEST_UNIT_ID);

        const service = new CollaborationUndoRedoService(
            instanceService as unknown as IUniverInstanceService,
            commandService as unknown as ICommandService,
            contextService as unknown as IContextService,
            rpcService,
            logService
        );

        // Push an undo/redo item
        service.pushUndoRedo({
            unitID: TEST_UNIT_ID,
            undoMutations: [createSetRangeValuesMutation()],
            redoMutations: [createSetRangeValuesMutation()],
        });

        // Simulate local mutation (fromCollab: false or undefined)
        const localMutation = createInsertRowMutation();
        commandService.emitMutationExecutedForCollab(
            {
                id: localMutation.id,
                type: CommandType.MUTATION,
                params: localMutation.params,
            },
            { fromCollab: false }
        );

        // Stack should not be marked dirty for local mutations
        // The transform service should not be called
        expect(mockTransformService.transformUndoStack).not.toHaveBeenCalled();

        service.dispose();
    });

    it('clears dirty state on clearUndoRedo', () => {
        const instanceService = new MockUniverInstanceService();
        const commandService = new MockCommandService();
        const contextService = new MockContextService();
        const rpcService = new MockRPCChannelService();
        const logService = new MockLogService();

        const mockTransformService = createMockUndoRedoTransformService();
        rpcService.setChannel(UNDO_REDO_TRANSFORM_SERVICE_NAME, mockTransformService);

        const unit = new MockUnit(TEST_UNIT_ID, UniverInstanceType.UNIVER_SHEET, 0);
        instanceService.addUnit(unit);
        instanceService.focusUnit(TEST_UNIT_ID);

        const service = new CollaborationUndoRedoService(
            instanceService as unknown as IUniverInstanceService,
            commandService as unknown as ICommandService,
            contextService as unknown as IContextService,
            rpcService,
            logService
        );

        // Push an undo/redo item
        service.pushUndoRedo({
            unitID: TEST_UNIT_ID,
            undoMutations: [createSetRangeValuesMutation()],
            redoMutations: [createSetRangeValuesMutation()],
        });

        // Simulate remote mutation to mark dirty
        const remoteMutation = createInsertRowMutation();
        commandService.emitMutationExecutedForCollab(
            {
                id: remoteMutation.id,
                type: CommandType.MUTATION,
                params: remoteMutation.params,
            },
            { fromCollab: true }
        );

        // Clear undo/redo
        service.clearUndoRedo(TEST_UNIT_ID);

        // Verify stack is empty
        const top = service.pitchTopUndoElement();
        expect(top).toBeNull();

        service.dispose();
    });

    it('handles missing unitId in mutation params', () => {
        const instanceService = new MockUniverInstanceService();
        const commandService = new MockCommandService();
        const contextService = new MockContextService();
        const rpcService = new MockRPCChannelService();
        const logService = new MockLogService();

        const mockTransformService = createMockUndoRedoTransformService();
        rpcService.setChannel(UNDO_REDO_TRANSFORM_SERVICE_NAME, mockTransformService);

        const service = new CollaborationUndoRedoService(
            instanceService as unknown as IUniverInstanceService,
            commandService as unknown as ICommandService,
            contextService as unknown as IContextService,
            rpcService,
            logService
        );

        // Simulate remote mutation without unitId
        commandService.emitMutationExecutedForCollab(
            {
                id: 'test-mutation',
                type: CommandType.MUTATION,
                params: {}, // No unitId
            },
            { fromCollab: true }
        );

        // Should not throw, just ignore the mutation
        service.dispose();
    });

    it('registers async undo/redo commands', () => {
        const instanceService = new MockUniverInstanceService();
        const commandService = new MockCommandService();
        const contextService = new MockContextService();
        const rpcService = new MockRPCChannelService();
        const logService = new MockLogService();

        const mockTransformService = createMockUndoRedoTransformService();
        rpcService.setChannel(UNDO_REDO_TRANSFORM_SERVICE_NAME, mockTransformService);

        const service = new CollaborationUndoRedoService(
            instanceService as unknown as IUniverInstanceService,
            commandService as unknown as ICommandService,
            contextService as unknown as IContextService,
            rpcService,
            logService
        );

        // Verify that async undo/redo commands are registered
        // The commands are registered in the constructor
        expect(commandService.hasCommand('univer.command.undo')).toBe(true);
        expect(commandService.hasCommand('univer.command.redo')).toBe(true);

        service.dispose();
    });
});
