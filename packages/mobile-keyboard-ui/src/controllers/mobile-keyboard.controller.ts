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

import { Disposable, DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY, DOCS_NORMAL_EDITOR_UNIT_ID_KEY, FOCUSING_FX_BAR_EDITOR, ICommandService, IContextService, Inject, Injector, IUniverInstanceService, UniverInstanceType } from '@univerjs/core';
import { DocSelectionRenderService, IEditorService } from '@univerjs/docs-ui';
import { getCurrentTypeOfRenderer, IRenderManagerService } from '@univerjs/engine-render';
import { BuiltInUIPart, connectInjector, ILayoutService, IMenuManagerService, IUIPartsService } from '@univerjs/ui';
import { distinctUntilChanged, filter } from 'rxjs';
import { KeyboardConfirmAndMoveOperation, KeyboardDeleteBackwardOperation, KeyboardInsertFunctionOperation, KeyboardInsertQuotesOperation, KeyboardInsertTextOperation, KeyboardSetModeOperation, KeyboardToggleKeyboardOperation } from '../commands/operations/keyboard.operation';
import { IMobileKeyboardService, KeyboardMode } from '../services/mobile-keyboard.service';
import { KeyboardContainer } from '../views/keyboard/common/KeyboardContainer';
import { MobileFormulaBar } from '../views/keyboard/common/MobileFormulaBar';
import { menuSchema } from './menu.schema';

export class MobileKeyboardController extends Disposable {
    constructor(
        @Inject(Injector) private readonly _injector: Injector,
        @IMobileKeyboardService private readonly _mobileKeyboardService: IMobileKeyboardService,
        @ICommandService private readonly _commandService: ICommandService,
        @IUIPartsService private readonly _uiPartsService: IUIPartsService,
        @ILayoutService private readonly _layoutService: ILayoutService,
        @IEditorService private readonly _editorService: IEditorService,
        @IContextService private readonly _contextService: IContextService,
        @IMenuManagerService private readonly _menuManagerService: IMenuManagerService
    ) {
        super();

        // Register the FAB component to be rendered
        // this.disposeWithMe(this._uiPartsService.registerComponent(BuiltInUIPart.GLOBAL, () => connectInjector(KeyboardFab, this._injector)));

        // Register the Keyboard Container component to be rendered
        // this.disposeWithMe(this._uiPartsService.registerComponent(BuiltInUIPart.TOOLBAR, () => connectInjector(OperationToolbar, this._injector)));
        this.disposeWithMe(this._uiPartsService.registerComponent(BuiltInUIPart.FOOTER, () => connectInjector(MobileFormulaBar, this._injector)));
        this.disposeWithMe(this._uiPartsService.registerComponent(BuiltInUIPart.CUSTOM_FOOTER, () => connectInjector(KeyboardContainer, this._injector)));

        // Replace the focus handler to ensure it is registered after the FAB component is registered
        this._initCommands();
        this._initMenus();
        this._initFocusHandler();
        this._initEditorListener();
        this._initKeyboardListener();
    }

    private _initCommands(): void {
        [
            KeyboardSetModeOperation,
            KeyboardInsertTextOperation,
            KeyboardDeleteBackwardOperation,
            KeyboardConfirmAndMoveOperation,
            KeyboardInsertFunctionOperation,
            KeyboardInsertQuotesOperation,
            KeyboardToggleKeyboardOperation,
        ].forEach((command) => this.disposeWithMe(this._commandService.registerCommand(command)));
    }

    private _initMenus(): void {
        // Register menu schema
        this._menuManagerService.mergeMenu(menuSchema);
    }

    private _initEditorListener(): void {
        this.disposeWithMe(this._editorService.editorAdd$.subscribe((editorUnitId) => {
            if (editorUnitId === DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY || editorUnitId === DOCS_NORMAL_EDITOR_UNIT_ID_KEY) {
                const editor = this._editorService.getEditor(editorUnitId);
                if (editor) {
                    editor.docSelectionRenderService.setInputMode('none');
                }
            }

            if (editorUnitId === DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY) {
                const formulaEditor = this._editorService.getEditor(DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY);
                this.disposeWithMe(formulaEditor!.focus$.subscribe(() => {
                    this._mobileKeyboardService.toggleKeyboard(true);
                }));
            }
        }));

        this.disposeWithMe(this._editorService.blur$.subscribe(() => {
            const editor = this._editorService.getEditor(DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY);
            if (editor) {
                editor.docSelectionRenderService.setInputMode('none');
            }
        }));

        this.disposeWithMe(this._editorService.focusEditorUnitId$
            .pipe(
                filter((focusEditorUnitId) => !!focusEditorUnitId),
                distinctUntilChanged()
            ).subscribe((focusEditorUnitId) => {
                const shouldToggleKeyboard = focusEditorUnitId === DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY || focusEditorUnitId === DOCS_NORMAL_EDITOR_UNIT_ID_KEY;
                // focus到非公式栏时，关闭键盘
                if (!shouldToggleKeyboard) {
                    this._mobileKeyboardService.toggleKeyboard(false);
                }
            }));
    }

    private _initKeyboardListener(): void {
        this.disposeWithMe(this._mobileKeyboardService.keyboardEnabled$.pipe(distinctUntilChanged()).subscribe((enabled) => {
            if (!enabled) {
                this._mobileKeyboardService.toggleKeyboard(false);
            }
        }));

        this.disposeWithMe(this._mobileKeyboardService.isKeyboardVisible$.pipe(distinctUntilChanged()).subscribe((isKeyboardVisible) => {
            if (isKeyboardVisible) {
                this._contextService.setContextValue(FOCUSING_FX_BAR_EDITOR, true);
                this._mobileKeyboardService.autoSelectMode();
                this._editorService.focus(DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY);
            } else {
                const formulaEditor = this._editorService.getEditor(DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY);
                if (formulaEditor) {
                    formulaEditor.docSelectionRenderService.setInputMode('none');
                }
                this._editorService.blur();
            }
        }));

        this.disposeWithMe(this._mobileKeyboardService.keyboardMode$.subscribe((mode) => {
            const editor = this._editorService.getEditor(DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY);
            if (!editor) return;
            if (mode === KeyboardMode.TEXT) {
                this._editorService.focus(DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY);
                editor.docSelectionRenderService.setInputMode('');
            } else {
                editor.docSelectionRenderService.setInputMode('none');
            }
        }));
    }

    private _initFocusHandler(): void {
        this.disposeWithMe(
            this._layoutService.registerFocusHandler(UniverInstanceType.UNIVER_SHEET, (_unitId: string) => {
                const renderManagerService = this._injector.get(IRenderManagerService);
                const instanceService = this._injector.get(IUniverInstanceService);
                const currentEditorRender = getCurrentTypeOfRenderer(UniverInstanceType.UNIVER_DOC, instanceService, renderManagerService);
                const docSelectionRenderService = currentEditorRender?.with(DocSelectionRenderService);
                docSelectionRenderService?.focus();
            }, true)
        );
    }
}
