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

import type { ICommand, IMutationInfo, IUndoRedoItem, IUndoRedoService } from '@univerjs/core';
import type { ISerializableUndoRedoItem, IUndoRedoTransformService } from './undo-redo-transform.service';
import {
    CommandType,
    DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY,
    DOCS_NORMAL_EDITOR_UNIT_ID_KEY,
    EDITOR_ACTIVATED,
    FOCUSING_FX_BAR_EDITOR,
    FOCUSING_SHEET,
    ICommandService,
    IContextService,
    ILogService,
    IUniverInstanceService,
    LocalUndoRedoService,
    RedoCommandId,
    sequenceExecute,
    UndoCommandId,
} from '@univerjs/core';
import { IRPCChannelService, toModule } from '@univerjs/rpc';
import { UNDO_REDO_TRANSFORM_SERVICE_NAME } from '../common/types';

/**
 * Accessor type for dependency injection (from @wendellhu/redi)
 * Defined locally to avoid direct dependency on redi package
 */
interface IAccessor {
    get<T>(token: any): T;
}

/**
 * Collaboration-aware undo/redo service with lazy transformation
 *
 * This service extends LocalUndoRedoService to handle OT transformation of
 * undo/redo stacks when remote mutations are received. Instead of transforming
 * immediately on every remote mutation (which is expensive), it uses a lazy
 * approach:
 *
 * 1. When remote mutation arrives: Mark stacks as "dirty" and accumulate mutations
 * 2. When user clicks undo/redo: Transform the relevant stack via remote RPC
 *
 * Benefits:
 * - Reduces CPU usage when user doesn't use undo/redo
 * - Offloads heavy WASM transforms to remote context
 * - Main thread stays responsive
 */
export class CollaborationUndoRedoService extends LocalUndoRedoService implements IUndoRedoService {
    private _undoRedoTransformService: IUndoRedoTransformService | null = null;
    private _initializationPromise: Promise<void> | null = null;

    /**
     * Tracks whether undo stacks are dirty (need transformation before use)
     */
    private readonly _dirtyUndoStacks = new Map<string, boolean>();

    /**
     * Tracks whether redo stacks are dirty (need transformation before use)
     */
    private readonly _dirtyRedoStacks = new Map<string, boolean>();

    /**
     * Accumulates remote mutations per document for batch transformation
     * Key: unitId, Value: array of mutation batches
     */
    private readonly _pendingRemoteMutations = new Map<string, IMutationInfo[][]>();

    constructor(
        @IUniverInstanceService _univerInstanceService: IUniverInstanceService,
        @ICommandService _commandService: ICommandService,
        @IContextService private readonly _collab_contextService: IContextService,
        @IRPCChannelService private readonly _rpcChannelService: IRPCChannelService,
        @ILogService private readonly _logService: ILogService
    ) {
        super(_univerInstanceService, _commandService, _collab_contextService);

        this._initRemoteService();
        this._initMutationListener();
        this._registerAsyncCommands();
    }

    /**
     * Initialize connection to remote's undo-redo transform service
     */
    private _initRemoteService(): void {
        this._initializationPromise = this._initRemoteServiceAsync();
    }

    private async _initRemoteServiceAsync(): Promise<void> {
        try {
            this._undoRedoTransformService = toModule<IUndoRedoTransformService>(
                this._rpcChannelService.requestChannel(UNDO_REDO_TRANSFORM_SERVICE_NAME)
            );
            this._logService.log('CollaborationUndoRedoService: Connected to remote transform service');
        } catch (error) {
            this._logService.error('CollaborationUndoRedoService: Failed to connect to remote transform service', error);
        }
    }

    /**
     * Listen for remote mutations and mark stacks as dirty
     */
    private _initMutationListener(): void {
        this.disposeWithMe(this._commandService.onMutationExecutedForCollab((command, options) => {
            // Only process remote mutations (fromCollab: true)
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

            // Mark stacks as dirty - they need transformation before use
            this._markStacksDirty(unitId);

            // Accumulate the remote mutation for later transformation
            this._accumulateRemoteMutation(unitId, {
                id: command.id,
                type: CommandType.MUTATION,
                params,
            });
        }));
    }

    /**
     * Register async undo/redo commands that handle transformation
     */
    private _registerAsyncCommands(): void {
        // Create async undo command that transforms before execution
        const AsyncUndoCommand: ICommand = {
            id: UndoCommandId,
            type: CommandType.COMMAND,
            handler: async (accessor: IAccessor) => {
                return this._handleAsyncUndo(accessor);
            },
        };

        // Create async redo command that transforms before execution
        const AsyncRedoCommand: ICommand = {
            id: RedoCommandId,
            type: CommandType.COMMAND,
            handler: async (accessor: IAccessor) => {
                return this._handleAsyncRedo(accessor);
            },
        };

        // Register our async commands (overrides the ones from LocalUndoRedoService)
        this._commandService.unregisterCommand(UndoCommandId);
        this._commandService.unregisterCommand(RedoCommandId);
        this.disposeWithMe(this._commandService.registerCommand(AsyncUndoCommand));
        this.disposeWithMe(this._commandService.registerCommand(AsyncRedoCommand));
    }

    /**
     * Handle async undo with lazy transformation
     */
    private async _handleAsyncUndo(accessor: IAccessor): Promise<boolean> {
        const unitId = this._getFocusedUnitIdInternal();
        if (!unitId) {
            return false;
        }

        // If stack is dirty, transform it first
        if (this._dirtyUndoStacks.get(unitId)) {
            const transformed = await this._transformUndoStack(unitId);
            if (!transformed) {
                this._logService.warn(`CollaborationUndoRedoService: Failed to transform undo stack for ${unitId}`);
                // Don't clear, let user retry
                return false;
            }
            this._dirtyUndoStacks.set(unitId, false);
        }

        // Now proceed with normal undo
        const element = this.pitchTopUndoElement();
        if (!element) {
            return false;
        }

        const commandService = accessor.get<ICommandService>(ICommandService);
        const result = sequenceExecute(element.undoMutations, commandService);
        if (result) {
            this.popUndoToRedo();
            return true;
        }

        return false;
    }

    /**
     * Handle async redo with lazy transformation
     */
    private async _handleAsyncRedo(accessor: IAccessor): Promise<boolean> {
        const unitId = this._getFocusedUnitIdInternal();
        if (!unitId) {
            return false;
        }

        // If stack is dirty, transform it first
        if (this._dirtyRedoStacks.get(unitId)) {
            const transformed = await this._transformRedoStack(unitId);
            if (!transformed) {
                this._logService.warn(`CollaborationUndoRedoService: Failed to transform redo stack for ${unitId}`);
                // Don't clear, let user retry
                return false;
            }
            this._dirtyRedoStacks.set(unitId, false);
        }

        // Now proceed with normal redo
        const element = this.pitchTopRedoElement();
        if (!element) {
            return false;
        }

        const commandService = accessor.get<ICommandService>(ICommandService);
        const result = sequenceExecute(element.redoMutations, commandService);
        if (result) {
            this.popRedoToUndo();
            return true;
        }

        return false;
    }

    /**
     * Get the focused unit ID (replicates base class private method logic)
     */
    private _getFocusedUnitIdInternal(): string {
        let unitID: string = '';

        const isFocusSheet = this._collab_contextService.getContextValue(FOCUSING_SHEET);
        const isFocusFormulaEditor = this._collab_contextService.getContextValue(FOCUSING_FX_BAR_EDITOR);
        const isFocusEditor = this._collab_contextService.getContextValue(EDITOR_ACTIVATED);

        if (isFocusSheet) {
            if (isFocusFormulaEditor) {
                unitID = DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY;
            } else if (isFocusEditor) {
                unitID = DOCS_NORMAL_EDITOR_UNIT_ID_KEY;
            } else {
                unitID = this._univerInstanceService.getFocusedUnit()?.getUnitId() ?? '';
            }
        } else {
            unitID = this._univerInstanceService.getFocusedUnit()?.getUnitId() ?? '';
        }

        return unitID;
    }

    /**
     * Mark both undo and redo stacks as dirty for a document
     */
    private _markStacksDirty(unitId: string): void {
        this._dirtyUndoStacks.set(unitId, true);
        this._dirtyRedoStacks.set(unitId, true);
    }

    /**
     * Accumulate a remote mutation for later batch transformation
     */
    private _accumulateRemoteMutation(unitId: string, mutation: IMutationInfo): void {
        let pending = this._pendingRemoteMutations.get(unitId);
        if (!pending) {
            pending = [];
            this._pendingRemoteMutations.set(unitId, pending);
        }
        // Add as a new batch (each remote changeset is a batch)
        pending.push([mutation]);
    }

    /**
     * Transform the undo stack via remote RPC
     */
    private async _transformUndoStack(unitId: string): Promise<boolean> {
        const remoteMutations = this._getAndClearPendingMutations(unitId);
        if (!remoteMutations.length) {
            return true; // Nothing to transform against
        }

        const undoStack = this._getUndoStack(unitId);
        if (!undoStack?.length) {
            return true; // No undo stack to transform
        }

        // Ensure remote service is ready
        if (this._initializationPromise) {
            await this._initializationPromise;
        }

        if (!this._undoRedoTransformService) {
            this._logService.error('CollaborationUndoRedoService: Remote transform service not available');
            return false;
        }

        try {
            // Convert to serializable format
            const serializableStack = this._toSerializableStack(undoStack);
            const flatMutations = remoteMutations.flat();

            // Call remote to transform
            const result = await this._undoRedoTransformService.transformUndoStack(
                unitId,
                serializableStack,
                flatMutations
            );

            if (!result.success || !result.items) {
                this._logService.error(`CollaborationUndoRedoService: Transform failed: ${result.error}`);
                return false;
            }

            // Apply transformed items back to the stack
            this._applyTransformedStack(undoStack, result.items);

            this._logService.log(
                `CollaborationUndoRedoService: Transformed ${undoStack.length} undo items for ${unitId} against ${flatMutations.length} remote mutations`
            );

            return true;
        } catch (error) {
            this._logService.error('CollaborationUndoRedoService: Exception during undo transform', error);
            return false;
        }
    }

    /**
     * Transform the redo stack via remote RPC
     */
    private async _transformRedoStack(unitId: string): Promise<boolean> {
        const remoteMutations = this._getAndClearPendingMutations(unitId);
        if (!remoteMutations.length) {
            return true; // Nothing to transform against
        }

        const redoStack = this._getRedoStack(unitId);
        if (!redoStack?.length) {
            return true; // No redo stack to transform
        }

        // Ensure remote service is ready
        if (this._initializationPromise) {
            await this._initializationPromise;
        }

        if (!this._undoRedoTransformService) {
            this._logService.error('CollaborationUndoRedoService: Remote transform service not available');
            return false;
        }

        try {
            // Convert to serializable format
            const serializableStack = this._toSerializableStack(redoStack);
            const flatMutations = remoteMutations.flat();

            // Call remote to transform
            const result = await this._undoRedoTransformService.transformRedoStack(
                unitId,
                serializableStack,
                flatMutations
            );

            if (!result.success || !result.items) {
                this._logService.error(`CollaborationUndoRedoService: Transform failed: ${result.error}`);
                return false;
            }

            // Apply transformed items back to the stack
            this._applyTransformedStack(redoStack, result.items);

            this._logService.log(
                `CollaborationUndoRedoService: Transformed ${redoStack.length} redo items for ${unitId} against ${flatMutations.length} remote mutations`
            );

            return true;
        } catch (error) {
            this._logService.error('CollaborationUndoRedoService: Exception during redo transform', error);
            return false;
        }
    }

    /**
     * Get and clear pending remote mutations for a document
     */
    private _getAndClearPendingMutations(unitId: string): IMutationInfo[][] {
        const pending = this._pendingRemoteMutations.get(unitId);
        if (!pending) {
            return [];
        }
        this._pendingRemoteMutations.delete(unitId);
        return pending;
    }

    /**
     * Convert IUndoRedoItem[] to serializable format for RPC
     */
    private _toSerializableStack(stack: IUndoRedoItem[]): ISerializableUndoRedoItem[] {
        return stack.map((item) => ({
            unitID: item.unitID,
            undoMutations: item.undoMutations,
            redoMutations: item.redoMutations,
        }));
    }

    /**
     * Apply transformed items back to the original stack
     */
    private _applyTransformedStack(
        originalStack: IUndoRedoItem[],
        transformedItems: ISerializableUndoRedoItem[]
    ): void {
        for (let i = 0; i < originalStack.length && i < transformedItems.length; i++) {
            originalStack[i].undoMutations = transformedItems[i].undoMutations;
            originalStack[i].redoMutations = transformedItems[i].redoMutations;
        }
    }

    /**
     * Override clearUndoRedo to also clear dirty state
     */
    override clearUndoRedo(unitId: string): void {
        super.clearUndoRedo(unitId);
        this._dirtyUndoStacks.delete(unitId);
        this._dirtyRedoStacks.delete(unitId);
        this._pendingRemoteMutations.delete(unitId);
    }

    override dispose(): void {
        super.dispose();
        this._dirtyUndoStacks.clear();
        this._dirtyRedoStacks.clear();
        this._pendingRemoteMutations.clear();
    }
}
