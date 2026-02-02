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
import { createIdentifier, Disposable, ILogService } from '@univerjs/core';
import { ITransformService } from './transform.service';

/**
 * Serializable undo/redo item for RPC transfer
 */
export interface ISerializableUndoRedoItem {
    unitID: string;
    undoMutations: IMutationInfo[];
    redoMutations: IMutationInfo[];
    id?: string;
}

/**
 * Result of transforming undo/redo stacks
 */
export interface ITransformStackResult {
    success: boolean;
    items?: ISerializableUndoRedoItem[];
    error?: string;
}

/**
 * Service interface for transforming undo/redo stacks in the remote context
 *
 * This service is called by the client/main thread when the user performs
 * an undo/redo action and the stacks are marked as dirty (need transform).
 */
export interface IUndoRedoTransformService {
    /**
     * Transform an undo stack against remote mutations
     *
     * @param unitId Document ID
     * @param undoStack The current undo stack items
     * @param remoteMutations Remote mutations to transform against
     * @returns Transformed undo stack or error
     */
    transformUndoStack(
        unitId: string,
        undoStack: ISerializableUndoRedoItem[],
        remoteMutations: IMutationInfo[]
    ): Promise<ITransformStackResult>;

    /**
     * Transform a redo stack against remote mutations
     *
     * @param unitId Document ID
     * @param redoStack The current redo stack items
     * @param remoteMutations Remote mutations to transform against
     * @returns Transformed redo stack or error
     */
    transformRedoStack(
        unitId: string,
        redoStack: ISerializableUndoRedoItem[],
        remoteMutations: IMutationInfo[]
    ): Promise<ITransformStackResult>;

    /**
     * Transform both stacks at once (more efficient if both need transform)
     *
     * @param unitId Document ID
     * @param undoStack The current undo stack items
     * @param redoStack The current redo stack items
     * @param remoteMutations Remote mutations to transform against
     * @returns Transformed stacks or error
     */
    transformBothStacks(
        unitId: string,
        undoStack: ISerializableUndoRedoItem[],
        redoStack: ISerializableUndoRedoItem[],
        remoteMutations: IMutationInfo[]
    ): Promise<{
        undoResult: ITransformStackResult;
        redoResult: ITransformStackResult;
    }>;
}

export const IUndoRedoTransformService = createIdentifier<IUndoRedoTransformService>(
    'univer.collaboration.remote.undo-redo-transform.service'
);

/**
 * Implementation of undo/redo transform service
 *
 * Runs in the remote context and handles lazy transformation of
 * undo/redo stacks when the user performs an undo/redo action.
 */
export class UndoRedoTransformService extends Disposable implements IUndoRedoTransformService {
    constructor(
        @ITransformService private readonly _transformService: ITransformService,
        @ILogService private readonly _logger: ILogService
    ) {
        super();
    }

    /**
     * Transform an undo stack against remote mutations
     */
    async transformUndoStack(
        unitId: string,
        undoStack: ISerializableUndoRedoItem[],
        remoteMutations: IMutationInfo[]
    ): Promise<ITransformStackResult> {
        return this._transformStack(unitId, undoStack, remoteMutations, 'undo');
    }

    /**
     * Transform a redo stack against remote mutations
     */
    async transformRedoStack(
        unitId: string,
        redoStack: ISerializableUndoRedoItem[],
        remoteMutations: IMutationInfo[]
    ): Promise<ITransformStackResult> {
        return this._transformStack(unitId, redoStack, remoteMutations, 'redo');
    }

    /**
     * Transform both stacks at once
     */
    async transformBothStacks(
        unitId: string,
        undoStack: ISerializableUndoRedoItem[],
        redoStack: ISerializableUndoRedoItem[],
        remoteMutations: IMutationInfo[]
    ): Promise<{
        undoResult: ITransformStackResult;
        redoResult: ITransformStackResult;
    }> {
        const [undoResult, redoResult] = await Promise.all([
            this._transformStack(unitId, undoStack, remoteMutations, 'undo'),
            this._transformStack(unitId, redoStack, remoteMutations, 'redo'),
        ]);

        return { undoResult, redoResult };
    }

    /**
     * Internal method to transform a stack
     */
    private async _transformStack(
        unitId: string,
        stack: ISerializableUndoRedoItem[],
        remoteMutations: IMutationInfo[],
        stackType: 'undo' | 'redo'
    ): Promise<ITransformStackResult> {
        if (!remoteMutations.length) {
            return { success: true, items: stack };
        }

        if (!stack.length) {
            return { success: true, items: [] };
        }

        try {
            const transformedItems: ISerializableUndoRedoItem[] = [];

            for (const item of stack) {
                // Transform undo mutations
                const undoResult = await this._transformService.transformList(
                    item.undoMutations,
                    remoteMutations
                );

                if (undoResult.error) {
                    this._logger.error(
                        `UndoRedoTransformService: Failed to transform ${stackType} undo mutations for ${unitId}: ${undoResult.error}`
                    );
                    return { success: false, error: undoResult.error };
                }

                // Transform redo mutations
                const redoResult = await this._transformService.transformList(
                    item.redoMutations,
                    remoteMutations
                );

                if (redoResult.error) {
                    this._logger.error(
                        `UndoRedoTransformService: Failed to transform ${stackType} redo mutations for ${unitId}: ${redoResult.error}`
                    );
                    return { success: false, error: redoResult.error };
                }

                transformedItems.push({
                    unitID: item.unitID,
                    undoMutations: undoResult.m1Primes,
                    redoMutations: redoResult.m1Primes,
                    id: item.id,
                });
            }

            this._logger.log(
                `UndoRedoTransformService: Transformed ${stack.length} ${stackType} items for ${unitId} against ${remoteMutations.length} remote mutations`
            );

            return { success: true, items: transformedItems };
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);
            this._logger.error(
                `UndoRedoTransformService: Exception transforming ${stackType} stack for ${unitId}: ${errorMessage}`
            );
            return { success: false, error: errorMessage };
        }
    }
}
