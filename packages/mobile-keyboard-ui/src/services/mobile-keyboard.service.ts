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

import type { IRange, Workbook } from '@univerjs/core';
import type { IScrollToCellOperationParams } from '@univerjs/sheets';
import type { IInsertFunctionOperationParams } from '@univerjs/sheets-formula-ui';
import type { IEditorBridgeService, IEditorBridgeServiceVisibleParam, IMoveSelectionEnterAndTabCommandParams } from '@univerjs/sheets-ui';
import type { Observable } from 'rxjs';
import { CellValueType, createIdentifier, Direction, Disposable, DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY, FOCUSING_FX_BAR_EDITOR, ICommandService, IContextService, IUniverInstanceService, UniverInstanceType } from '@univerjs/core';
import { DeleteLeftCommand, IEditorService } from '@univerjs/docs-ui';
import { DeviceInputEventType } from '@univerjs/engine-render';
import {
    ScrollToCellOperation,
} from '@univerjs/sheets';
import { InsertFunctionOperation } from '@univerjs/sheets-formula-ui';
import { IEditorBridgeService as IEditorBridgeServiceToken, MoveSelectionEnterAndTabCommand, SetCellEditVisibleOperation } from '@univerjs/sheets-ui';
import { KeyCode } from '@univerjs/ui';
import { BehaviorSubject, distinctUntilChanged } from 'rxjs';

export enum KeyboardMode {
    FORMULA = 'formula',
    NUMBER = 'number',
    TEXT = 'text',
}

export interface IMobileKeyboardService {
    /** Mobile keyboard UI visibility (NOT the same as editor visibility) */
    isKeyboardVisible$: Observable<boolean>;
    /** Show the mobile keyboard UI */
    showKeyboard(): void;
    /** Hide the mobile keyboard UI */
    hideKeyboard(): void;

    /** Active keyboard mode */
    keyboardMode$: Observable<KeyboardMode>;
    /** Set the active keyboard mode */
    setMode(mode: KeyboardMode): void;

    /** Insert text into the FormulaBar editor at current cursor position */
    insertText(text: string): void;
    /** Insert function into the FormulaBar editor at current cursor position */
    insertFunction(funcName: string): void;
    /** Insert quotes into the FormulaBar editor at current cursor position */
    insertQuotes(quotes: string): void;
    /** Delete character before cursor */
    deleteBackward(): void;
    /** Confirm input and move to next cell */
    confirmAndMove(direction: Direction.DOWN | Direction.RIGHT): void;
}

export const IMobileKeyboardService = createIdentifier<IMobileKeyboardService>('mobile-keyboard-ui.mobile-keyboard.service');

export class MobileKeyboardService extends Disposable implements IMobileKeyboardService {
    private readonly _isKeyboardVisible$ = new BehaviorSubject<boolean>(false);
    readonly isKeyboardVisible$ = this._isKeyboardVisible$.asObservable();

    private readonly _keyboardMode$ = new BehaviorSubject<KeyboardMode>(KeyboardMode.NUMBER);
    readonly keyboardMode$ = this._keyboardMode$.asObservable();

    private _lastUsedMode: KeyboardMode = KeyboardMode.NUMBER;

    constructor(
        @IContextService private readonly _contextService: IContextService,
        @ICommandService private readonly _commandService: ICommandService,
        @IUniverInstanceService private readonly _univerInstanceService: IUniverInstanceService,
        @IEditorBridgeServiceToken private readonly _editorBridgeService: IEditorBridgeService,
        @IEditorService private readonly _editorService: IEditorService
    ) {
        super();

        this.disposeWithMe(this.isKeyboardVisible$.pipe(distinctUntilChanged()).subscribe((visible) => {
            if (!visible) return;
            this._autoSelectMode();
            const workbook = this._univerInstanceService.getCurrentUnitForType<Workbook>(UniverInstanceType.UNIVER_SHEET);
            if (!workbook) {
                return;
            }

            // When clicking on the formula bar, the cell editor also needs to enter the edit state
            const visibleState = this._editorBridgeService.isVisible();
            const editState = this._editorBridgeService.getEditCellState();
            if (visibleState.visible === false) {
                this._commandService.syncExecuteCommand(
                    SetCellEditVisibleOperation.id,
                    {
                        visible: true,
                        eventType: DeviceInputEventType.PointerDown,
                        unitId: editState!.unitId,
                    } as IEditorBridgeServiceVisibleParam
                );
            }

             // Open the normal editor first, and then we mark formula editor as activated.
            this._contextService.setContextValue(FOCUSING_FX_BAR_EDITOR, true);

            setTimeout(() => {
                this._commandService.executeCommand(ScrollToCellOperation.id, {
                    range: {
                        startRow: editState!.row,
                        startColumn: editState!.column,
                        endRow: editState!.row,
                        endColumn: editState!.column,
                    } as IRange,
                    unitId: editState!.unitId,
                } satisfies IScrollToCellOperationParams);
            }, 100);
        }));
    }

    private _autoSelectMode(): void {
        const editState = this._editorBridgeService.getEditCellState();
        if (!editState) {
            this.setMode(this._lastUsedMode);
            return;
        }

        const workbook = this._univerInstanceService.getUnit<Workbook>(editState.unitId);
        const sheet = workbook?.getSheetBySheetId(editState.sheetId);
        const cell = sheet?.getCell(editState.row, editState.column);
        if (cell?.f) {
            this.setMode(KeyboardMode.FORMULA);
            return;
        }
        if (cell?.t === CellValueType.NUMBER) {
            this.setMode(KeyboardMode.NUMBER);
            return;
        }
        if (cell?.t === CellValueType.STRING) {
            this.setMode(KeyboardMode.TEXT);
            return;
        }
        this.setMode(KeyboardMode.FORMULA);
    }

    showKeyboard(): void {
        this._isKeyboardVisible$.next(true);
    }

    hideKeyboard(): void {
        this._isKeyboardVisible$.next(false);
    }

    setMode(mode: KeyboardMode): void {
        this._keyboardMode$.next(mode);
        this._lastUsedMode = mode;
    }

    insertText(text: string): void {
        // When clicking on the formula bar, the cell editor also needs to enter the edit state
        const visibleState = this._editorBridgeService.isVisible();
        const editState = this._editorBridgeService.getEditCellState();
        if (visibleState.visible === false) {
            this._commandService.syncExecuteCommand(
                SetCellEditVisibleOperation.id,
                {
                    visible: true,
                    eventType: DeviceInputEventType.Keyboard,
                    unitId: editState!.unitId,
                } as IEditorBridgeServiceVisibleParam
            );
        }

        // Open the normal editor first, and then we mark formula editor as activated.
        this._contextService.setContextValue(FOCUSING_FX_BAR_EDITOR, true);

        const formulaEditor = this._editorService.getEditor(DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY);
        if (!formulaEditor) {
            return;
        }
        formulaEditor.docSelectionRenderService.setInputContent(text);
        formulaEditor.docSelectionRenderService.dispatchInputDomEvent(new InputEvent('input', {
            data: text,
            inputType: 'insertText',
        }));
    }

    insertFunction(funcName: string): void {
        this._commandService.executeCommand(InsertFunctionOperation.id, { value: funcName } satisfies IInsertFunctionOperationParams);
    }

    insertQuotes(quotes: string): void {
        this.insertText(quotes);
    }

    deleteBackward() {
        this._commandService.executeCommand(DeleteLeftCommand.id);
    }

    confirmAndMove(direction: Direction.DOWN | Direction.RIGHT = Direction.DOWN): void {
        const workbook = this._univerInstanceService.getCurrentUnitOfType<Workbook>(UniverInstanceType.UNIVER_SHEET);
        const editState = this._editorBridgeService.getEditCellState();

        if (!workbook || !editState?.unitId) {
            return;
        }
        const keycode = direction === Direction.RIGHT ? KeyCode.TAB : KeyCode.ENTER;
        if (this._editorBridgeService.isVisible().visible) {
            this._editorBridgeService.disableForceKeepVisible();
            this._commandService.syncExecuteCommand(SetCellEditVisibleOperation.id, {
                visible: false,
                eventType: DeviceInputEventType.Keyboard,
                keycode,
                unitId: editState.unitId,
            } satisfies IEditorBridgeServiceVisibleParam);
        } else {
            this._commandService.executeCommand(MoveSelectionEnterAndTabCommand.id, {
                direction,
                keycode,
            } satisfies IMoveSelectionEnterAndTabCommandParams);
        }
    }
}
