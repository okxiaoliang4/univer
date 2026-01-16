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

import type { IMutationInfo, IUndoRedoItem, IUndoRedoService } from '@univerjs/core';
import { CommandType, ICommandService, IContextService, ILogService, IUniverInstanceService, LocalUndoRedoService } from '@univerjs/core';
import { ITransformService } from './transform.service';

export class CollaborationUndoRedoService extends LocalUndoRedoService implements IUndoRedoService {
    constructor(
        @IUniverInstanceService _univerInstanceService: IUniverInstanceService,
        @ICommandService _commandService: ICommandService,
        @IContextService _contextService: IContextService,
        @ITransformService private readonly _transformService: ITransformService,
        @ILogService private readonly _logService: ILogService
    ) {
        super(_univerInstanceService, _commandService, _contextService);

        this.disposeWithMe(this._commandService.onMutationExecutedForCollab((command, options) => {
            if (!options?.fromCollab) {
                return;
            }

            if (command.type !== CommandType.MUTATION) {
                return;
            }

            const params = command.params as { unitId?: string } | undefined;
            const unitId = params?.unitId;
            if (!unitId || !params) {
                return;
            }

            this._transformStacks(unitId, [{
                id: command.id,
                type: CommandType.MUTATION,
                params,
            }]);
        }));
    }

    private _transformStacks(unitId: string, remoteMutations: IMutationInfo[]): void {
        if (!remoteMutations.length) {
            return;
        }

        const undoStack = this._getUndoStack(unitId);
        const redoStack = this._getRedoStack(unitId);

        if (!undoStack?.length && !redoStack?.length) {
            return;
        }

        const transformList = (localMutations: IMutationInfo[]): IMutationInfo[] | null => {
            if (localMutations.length === 0) {
                return localMutations;
            }

            const result = this._transformService.transformList(localMutations, remoteMutations);
            if (result.error) {
                this._logService.error(
                    `[CollaborationUndoRedoService] OT transform failed for unitId ${unitId}: ${result.error}`
                );
                return null;
            }

            return result.m1Primes;
        };

        const applyTransform = (stack: IUndoRedoItem[]): boolean => {
            for (const item of stack) {
                const nextUndo = transformList(item.undoMutations);
                if (!nextUndo) {
                    return false;
                }

                const nextRedo = transformList(item.redoMutations);
                if (!nextRedo) {
                    return false;
                }

                item.undoMutations = nextUndo;
                item.redoMutations = nextRedo;
            }

            return true;
        };

        if (undoStack?.length && !applyTransform(undoStack)) {
            this.clearUndoRedo(unitId);
            return;
        }

        if (redoStack?.length && !applyTransform(redoStack)) {
            this.clearUndoRedo(unitId);
        }
    }
}
