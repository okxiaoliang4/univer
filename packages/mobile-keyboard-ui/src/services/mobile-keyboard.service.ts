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

import type { Workbook } from '@univerjs/core';
import type { IInsertFunctionOperationParams } from '@univerjs/sheets-formula-ui';
import type { IEditorBridgeService, IEditorBridgeServiceVisibleParam, IMoveSelectionEnterAndTabCommandParams } from '@univerjs/sheets-ui';
import type { Observable } from 'rxjs';
import { CellValueType, createIdentifier, createInternalEditorID, Direction, Disposable, FOCUSING_FX_BAR_EDITOR, ICommandService, IContextService, IUniverInstanceService, UniverInstanceType } from '@univerjs/core';
import { DeleteLeftCommand, IEditorService } from '@univerjs/docs-ui';
import { DeviceInputEventType } from '@univerjs/engine-render';
import { InsertFunctionOperation } from '@univerjs/sheets-formula-ui';
import { IEditorBridgeService as IEditorBridgeServiceToken, MoveSelectionEnterAndTabCommand, SetCellEditVisibleOperation } from '@univerjs/sheets-ui';
import { IDialogService, ISidebarService, KeyCode } from '@univerjs/ui';
import { BehaviorSubject, combineLatest, distinctUntilChanged, map, startWith } from 'rxjs';

export enum KeyboardMode {
    FORMULA = 'formula',
    NUMBER = 'number',
    TEXT = 'text',
}

export interface IMobileKeyboardService {
    /** Whether the mobile keyboard is enabled */
    keyboardEnabled$: Observable<boolean>;

    /** Mobile keyboard UI visibility (NOT the same as editor visibility) */
    isKeyboardVisible$: Observable<boolean>;
    /** Toggle the mobile keyboard UI */
    toggleKeyboard(visible?: boolean): void;

    /** Auto select mode */
    autoSelectMode(): void;

    /** Active keyboard mode */
    keyboardMode$: Observable<KeyboardMode>;
    /** Get the active keyboard mode */
    getKeyboardMode(): KeyboardMode;
    /** Set the active keyboard mode */
    setKeyboardMode(mode: KeyboardMode): void;

    /** Insert text into the FormulaBar editor at current cursor position */
    insertText(text: string): void;
    /** Insert function into the FormulaBar editor at current cursor position */
    insertFunction(funcName: string): void;
    /** Insert quotes into the FormulaBar editor at current cursor position */
    insertQuotes(quotes: string): void;
    /** Delete character before cursor */
    deleteBackward(): void;
    /** Toggle negative sign */
    negativeNumber(): void;
    /** Confirm input and move to next cell */
    confirmAndMove(direction: Direction.DOWN | Direction.RIGHT): void;
}

export const IMobileKeyboardService = createIdentifier<IMobileKeyboardService>('mobile-keyboard-ui.mobile-keyboard.service');

export class MobileKeyboardService extends Disposable implements IMobileKeyboardService {
    readonly keyboardEnabled$: Observable<boolean>;

    private readonly _isKeyboardVisible$ = new BehaviorSubject<boolean>(false);
    readonly isKeyboardVisible$ = this._isKeyboardVisible$.asObservable();

    private readonly _keyboardMode$ = new BehaviorSubject<KeyboardMode>(KeyboardMode.FORMULA);
    readonly keyboardMode$ = this._keyboardMode$.asObservable();

    private _lastUsedMode: KeyboardMode = KeyboardMode.FORMULA;

    constructor(
        @ICommandService private readonly _commandService: ICommandService,
        @IUniverInstanceService private readonly _univerInstanceService: IUniverInstanceService,
        @IEditorBridgeServiceToken private readonly _editorBridgeService: IEditorBridgeService,
        @IEditorService private readonly _editorService: IEditorService,
        @ISidebarService private readonly _sidebarService: ISidebarService,
        @IDialogService private readonly _dialogService: IDialogService,
        @IContextService private readonly _contextService: IContextService
    ) {
        super();

        this.keyboardEnabled$ = combineLatest([
            this._sidebarService.sidebarOptions$,
            this._dialogService.getDialogs$().pipe(startWith([])),
        ])
            .pipe(
                map(([sidebarOptions, dialogOptions]) => !sidebarOptions.visible && !dialogOptions.some((option) => option.open)),
                distinctUntilChanged()
            );
    }

    autoSelectMode(): void {
        const editor = this._editorService.getFocusEditor();
        const editorId = editor?.getEditorId();
        const isRichTextEditor = editorId?.startsWith(createInternalEditorID('RICH_TEXT_EDITOR'));
        if (isRichTextEditor) {
            this.setKeyboardMode(KeyboardMode.TEXT);
            return;
        }

        const editState = this._editorBridgeService.getEditCellState();
        if (!editState) {
            this.setKeyboardMode(this._lastUsedMode);
            return;
        }

        const workbook = this._univerInstanceService.getUnit<Workbook>(editState.unitId);
        const sheet = workbook?.getSheetBySheetId(editState.sheetId);
        const cell = sheet?.getCell(editState.row, editState.column);
        if (cell?.f) {
            this.setKeyboardMode(KeyboardMode.FORMULA);
            return;
        }
        if (cell?.t === CellValueType.NUMBER) {
            this.setKeyboardMode(KeyboardMode.NUMBER);
            return;
        }
        if (cell?.t === CellValueType.STRING) {
            this.setKeyboardMode(KeyboardMode.TEXT);
            return;
        }
        this.setKeyboardMode(KeyboardMode.FORMULA);
    }

    getVisible() {
        return this._isKeyboardVisible$.getValue();
    }

    toggleKeyboard(visible?: boolean): void {
        this._isKeyboardVisible$.next(visible === undefined ? !this._isKeyboardVisible$.getValue() : visible);
    }

    getKeyboardMode(): KeyboardMode {
        return this._keyboardMode$.getValue();
    }

    setKeyboardMode(mode: KeyboardMode): void {
        this._keyboardMode$.next(mode);
        this._lastUsedMode = mode;
    }

    insertText(text: string): void {
        const visibleState = this._editorBridgeService.isVisible();
        if (visibleState.visible === false) {
            this._commandService.syncExecuteCommand(
                SetCellEditVisibleOperation.id,
                {
                    visible: true,
                    eventType: DeviceInputEventType.PointerDown,
                    unitId: this._editorBridgeService.getEditCellState()!.unitId,
                } as IEditorBridgeServiceVisibleParam
            );

            // Open the normal editor first, and then we mark formula editor as activated.
            this._contextService.setContextValue(FOCUSING_FX_BAR_EDITOR, true);
        }

        const focusEditor = this._editorService.getFocusEditor();

        if (!focusEditor) {
            return;
        }
        const activeTextRange = focusEditor.docSelectionRenderService.getActiveTextRange();
        if (!activeTextRange) {
            const documentData = focusEditor?.getDocumentData();
            focusEditor.setSelectionRanges([{ startOffset: 0, endOffset: documentData?.body ? documentData.body.dataStream.length - 2 : 0 }]);
        }

        focusEditor.docSelectionRenderService.setInputContent(text);
        focusEditor.docSelectionRenderService.dispatchInputDomEvent(new InputEvent('input', {
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

    negativeNumber(): void {
        const editor = this._editorService.getFocusEditor();
        const documentData = editor?.getDocumentData();
        const num = Number(documentData?.body?.dataStream);
        if (Number.isNaN(num)) {
            return;
        }
        const newValue = (-num).toString();
        if (!editor) {
            return;
        }
        editor.setSelectionRanges([{ startOffset: 0, endOffset: documentData?.body ? documentData.body.dataStream.length - 2 : 0 }]);
        editor.docSelectionRenderService.setInputContent(newValue);
        editor.docSelectionRenderService.dispatchInputDomEvent(new InputEvent('input', {
            data: newValue,
            inputType: 'insertText',
        }));
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
