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

import type {
    IEditorBridgeServiceVisibleParam,
} from '@univerjs/sheets-ui';
import { FOCUSING_FX_BAR_EDITOR, ICommandService, IContextService, IUndoRedoService, LocaleService, RedoCommandId, UndoCommandId } from '@univerjs/core';
import { DeviceInputEventType } from '@univerjs/engine-render';
import { CopyIcon, CutIcon, DeleteIcon, KeyboardIcon, PasteSpecialDoubleIcon, RedoIcon, UndoIcon } from '@univerjs/icons';
import { ClearSelectionContentCommand } from '@univerjs/sheets';
import { IEditorBridgeService, SetCellEditVisibleOperation, SheetCopyCommand, SheetCutCommand, SheetPasteCommand } from '@univerjs/sheets-ui';
import { useDependency, useObservable } from '@univerjs/ui';
import { useCallback, useEffect, useState } from 'react';
import { KeyboardItem } from '../../..';
import { IMobileKeyboardService } from '../../../services/mobile-keyboard.service';

export function OperationToolbar() {
    const commandService = useDependency(ICommandService);
    const undoRedoService = useDependency(IUndoRedoService);
    const localeService = useDependency(LocaleService);
    const mobileKeyboardService = useDependency(IMobileKeyboardService);
    const editorBridgeService = useDependency(IEditorBridgeService);
    const contextService = useDependency(IContextService);
    const isKeyboardVisible = useObservable(mobileKeyboardService.isKeyboardVisible$, false);
    const isKeyboardEnabled = useObservable(mobileKeyboardService.keyboardEnabled$, true);
    const editState = useObservable(editorBridgeService.currentEditCellState$);

    const handleUndo = useCallback(() => {
        commandService.executeCommand(UndoCommandId);
    }, [commandService]);

    const handleRedo = useCallback(() => {
        commandService.executeCommand(RedoCommandId);
    }, [commandService]);

    const handleCopy = useCallback(() => {
        commandService.executeCommand(SheetCopyCommand.id);
    }, [commandService]);

    const handlePaste = useCallback(() => {
        commandService.executeCommand(SheetPasteCommand.id);
    }, [commandService]);

    const handleCut = useCallback(() => {
        commandService.executeCommand(SheetCutCommand.id);
    }, [commandService]);

    const handleClear = useCallback(() => {
        commandService.executeCommand(ClearSelectionContentCommand.id);
    }, [commandService]);

    const handleKeyboard = useCallback(() => {
        mobileKeyboardService.toggleKeyboard();
        if (!isKeyboardVisible) {
            // When clicking on the formula bar, the cell editor also needs to enter the edit state
            const visibleState = editorBridgeService.isVisible();
            if (visibleState.visible === false) {
                commandService.syncExecuteCommand(
                    SetCellEditVisibleOperation.id,
                    {
                        visible: true,
                        eventType: DeviceInputEventType.PointerDown,
                        unitId: editState!.unitId,
                    } as IEditorBridgeServiceVisibleParam
                );
            }

            // Open the normal editor first, and then we mark formula editor as activated.
            contextService.setContextValue(FOCUSING_FX_BAR_EDITOR, true);
        }
    }, [isKeyboardVisible, mobileKeyboardService, commandService, editorBridgeService, contextService, editState]);

    // Use observables to track undo/redo availability
    const [canUndo, setCanUndo] = useState(false);
    const [canRedo, setCanRedo] = useState(false);

    useEffect(() => {
        const subscriptions = [
            undoRedoService.undoRedoStatus$.subscribe((status) => {
                setCanUndo(status.undos > 0);
                setCanRedo(status.redos > 0);
            }),
        ];
        return () => {
            subscriptions.forEach((s) => s.unsubscribe());
        };
    }, [undoRedoService]);

    const t = (key: string) => localeService.t(key);

    return (
        <div
            className={`
              univer-flex univer-items-center univer-justify-between univer-border-b univer-border-gray-200 univer-px-1
              univer-py-2
              dark:!univer-border-gray-700 dark:!univer-bg-gray-800
            `}
        >
            <div className="univer-flex univer-flex-1 univer-gap-2">
                <KeyboardItem
                    onClick={handleUndo}
                    disabled={!canUndo}
                    size="small"
                    variant="text"
                >
                    <UndoIcon />
                </KeyboardItem>
                <KeyboardItem
                    onClick={handleRedo}
                    disabled={!canRedo}
                    variant="text"
                    size="small"
                >
                    <RedoIcon />
                </KeyboardItem>
                <KeyboardItem
                    onClick={handleCopy}
                    variant="text"
                    size="small"
                >
                    <CopyIcon />
                </KeyboardItem>
                <KeyboardItem
                    onClick={handlePaste}
                    variant="text"
                    size="small"
                >
                    <PasteSpecialDoubleIcon />
                </KeyboardItem>
                <KeyboardItem
                    onClick={handleCut}
                    variant="text"
                    size="small"
                >
                    <CutIcon />
                </KeyboardItem>
                <KeyboardItem
                    onClick={handleClear}
                    variant="danger"
                    size="small"
                >
                    <DeleteIcon />
                </KeyboardItem>
            </div>
            <div>
                <KeyboardItem
                    size="small"
                    onClick={handleKeyboard}
                    disabled={!isKeyboardEnabled}
                    variant={isKeyboardVisible ? 'primary' : 'text'}
                >
                    <KeyboardIcon />
                    {t('keyboard')}
                </KeyboardItem>
            </div>
        </div>
    );
}
