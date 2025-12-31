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

import type { IDeleteCommandParams, IInsertCommandParams } from '@univerjs/docs-ui';
import type { IMoveSelectionEnterAndTabCommandParams } from '../../../../commands/commands/set-selection.command';
import type { IEditorBridgeServiceVisibleParam } from '../../../../services/editor-bridge.service';
import { DeleteDirection, Direction, DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY, ICommandService, IUniverInstanceService, UniverInstanceType } from '@univerjs/core';
import { DocSelectionManagerService, SetTextSelectionsOperation } from '@univerjs/docs';
import { DeleteCommand, InsertCommand } from '@univerjs/docs-ui';
import { DeviceInputEventType, NORMAL_TEXT_SELECTION_PLUGIN_STYLE } from '@univerjs/engine-render';
import { KeyCode, useDependency, useObservable } from '@univerjs/ui';
import { useCallback } from 'react';
import {
    MoveSelectionEnterAndTabCommand,
} from '../../../../commands/commands/set-selection.command';
import { SetCellEditVisibleOperation } from '../../../../commands/operations/cell-edit.operation';
import {
    IEditorBridgeService,
} from '../../../../services/editor-bridge.service';

/**
 * Hook to handle keyboard input for mobile keyboards
 * Provides methods to insert text, delete, and navigate in the FormulaBar
 */
export function useKeyboardInput() {
    const commandService = useDependency(ICommandService);
    const docSelectionManagerService = useDependency(DocSelectionManagerService);
    const instanceService = useDependency(IUniverInstanceService);
    const workbook = instanceService.getCurrentUnitOfType(UniverInstanceType.UNIVER_SHEET);
    const editorBridgeService = useDependency(IEditorBridgeService);
    const editState = useObservable(editorBridgeService.currentEditCellState$);
    /**
     * Insert text into the FormulaBar editor at current cursor position
     */
    const insertText = useCallback(
        (text: string) => {
            if (!editorBridgeService.isVisible().visible) {
                commandService.syncExecuteCommand(SetCellEditVisibleOperation.id, {
                    visible: true,
                    eventType: DeviceInputEventType.Keyboard,
                    keycode: KeyCode.ENTER,
                    unitId: editState?.unitId,
                });
            }
            const activeRange = docSelectionManagerService.getActiveTextRange();
            const range = activeRange || {
                startOffset: 0,
                endOffset: 0,
                collapsed: true,
            };

            commandService.executeCommand(InsertCommand.id, {
                unitId: DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY,
                body: {
                    dataStream: text,
                },
                range,
            } satisfies IInsertCommandParams);
        },
        [commandService, docSelectionManagerService]
    );

    /**
     * Delete character before cursor
     */
    const deleteBackward = useCallback(() => {
        if (!editorBridgeService.isVisible().visible) {
            commandService.syncExecuteCommand(SetCellEditVisibleOperation.id, {
                visible: true,
                eventType: DeviceInputEventType.Keyboard,
                keycode: KeyCode.ENTER,
                unitId: editState?.unitId,
            });
        }
        const activeRange = docSelectionManagerService.getActiveTextRange();
        if (!activeRange) return;

        const { startOffset, endOffset, collapsed } = activeRange;

        if (collapsed) {
            if (startOffset === 0) return;
            commandService.executeCommand(DeleteCommand.id, {
                direction: DeleteDirection.LEFT,
                unitId: DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY,
                range: activeRange,
                len: 1,
            } satisfies IDeleteCommandParams);
        } else {
            commandService.executeCommand(DeleteCommand.id, {
                direction: DeleteDirection.LEFT,
                unitId: DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY,
                range: activeRange,
                len: endOffset - startOffset,
            } satisfies IDeleteCommandParams);
        }
    }, [commandService, docSelectionManagerService]);

    /**
     * Confirm input and move to next cell
     */
    const confirmAndMove = useCallback(
        (direction: Direction.DOWN | Direction.RIGHT = Direction.DOWN) => {
            if (!workbook || !editState?.unitId) {
                return;
            }
            const keycode = direction === Direction.RIGHT ? KeyCode.TAB : KeyCode.ENTER;
            if (editorBridgeService.isVisible().visible) {
                editorBridgeService.disableForceKeepVisible();
                commandService.syncExecuteCommand(SetCellEditVisibleOperation.id, {
                    visible: false,
                    eventType: DeviceInputEventType.Keyboard,
                    keycode,
                    unitId: editState.unitId,
                } satisfies IEditorBridgeServiceVisibleParam);
            } else {
                commandService.executeCommand(MoveSelectionEnterAndTabCommand.id, {
                    direction,
                    keycode,
                } satisfies IMoveSelectionEnterAndTabCommandParams);
            }
        },
        [commandService, workbook]
    );

    /**
     * Insert function and move cursor inside parentheses
     */
    const insertFunction = useCallback(
        (funcName: string) => {
            if (!editorBridgeService.isVisible().visible) {
                commandService.syncExecuteCommand(SetCellEditVisibleOperation.id, {
                    visible: true,
                    eventType: DeviceInputEventType.Keyboard,
                    keycode: KeyCode.ENTER,
                    unitId: editState?.unitId,
                });
            }
            const activeRange = docSelectionManagerService.getActiveTextRange();
            const startOffset = activeRange?.startOffset ?? 0;

            const functionText = `=${funcName}()`;
            commandService.executeCommand(InsertCommand.id, {
                unitId: DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY,
                body: {
                    dataStream: functionText,
                },
                range: activeRange || { startOffset: 0, endOffset: 0, collapsed: true },
            } satisfies IInsertCommandParams);

            // Move cursor inside parentheses
            const newPos = startOffset + functionText.length - 1;
            commandService.executeCommand(SetTextSelectionsOperation.id, {
                unitId: DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY,
                subUnitId: '',
                segmentId: '',
                isEditing: true,
                style: NORMAL_TEXT_SELECTION_PLUGIN_STYLE,
                ranges: [{
                    startOffset: newPos,
                    endOffset: newPos,
                    collapsed: true,
                }],
            });
        },
        [commandService, docSelectionManagerService]
    );

    /**
     * Insert double quotes and move cursor between them
     */
    const insertQuotes = useCallback(() => {
        const activeRange = docSelectionManagerService.getActiveTextRange();
        const startOffset = activeRange?.startOffset ?? 0;

        commandService.executeCommand(InsertCommand.id, {
            unitId: DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY,
            body: {
                dataStream: '""',
            },
            range: activeRange || { startOffset: 0, endOffset: 0, collapsed: true },
        } satisfies IInsertCommandParams);

        // Move cursor between quotes
        const newPos = startOffset + 1;
        commandService.executeCommand(SetTextSelectionsOperation.id, {
            unitId: DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY,
            subUnitId: '',
            segmentId: '',
            isEditing: true,
            style: NORMAL_TEXT_SELECTION_PLUGIN_STYLE,
            ranges: [{
                startOffset: newPos,
                endOffset: newPos,
                collapsed: true,
            }],
        });
    }, [commandService, docSelectionManagerService]);

    return {
        insertText,
        deleteBackward,
        confirmAndMove,
        insertFunction,
        insertQuotes,
    };
}
