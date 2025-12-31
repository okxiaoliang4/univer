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

import { Disposable, ICommandService, Inject, Injector, IUniverInstanceService, UniverInstanceType } from '@univerjs/core';
import { DocSelectionRenderService } from '@univerjs/docs-ui';
import { getCurrentTypeOfRenderer, IRenderManagerService } from '@univerjs/engine-render';
import { BuiltInUIPart, connectInjector, ILayoutService, IUIPartsService } from '@univerjs/ui';
import { KeyboardConfirmAndMoveOperation, KeyboardDeleteBackwardOperation, KeyboardHideOperation, KeyboardInsertFunctionOperation, KeyboardInsertQuotesOperation, KeyboardInsertTextOperation, KeyboardSetModeOperation, KeyboardShowOperation } from '../commands/operations/keyboard.operation';
import { IMobileKeyboardService } from '../services/mobile-keyboard.service';
import { KeyboardFab } from '../views/fab/KeyboardFab';
import { KeyboardContainer } from '../views/keyboard/common/KeyboardContainer';

export class MobileKeyboardController extends Disposable {
    constructor(
        @Inject(Injector) private readonly _injector: Injector,
        @IMobileKeyboardService private readonly _mobileKeyboardService: IMobileKeyboardService,
        @ICommandService private readonly _commandService: ICommandService,
        @IUIPartsService private readonly _uiPartsService: IUIPartsService,
        @ILayoutService private readonly _layoutService: ILayoutService
    ) {
        super();

        // Register the FAB component to be rendered
        this.disposeWithMe(this._uiPartsService.registerComponent(BuiltInUIPart.GLOBAL, () => connectInjector(KeyboardFab, this._injector)));

        // Register the Keyboard Container component to be rendered
        this.disposeWithMe(this._uiPartsService.registerComponent(BuiltInUIPart.FOOTER, () => connectInjector(KeyboardContainer, this._injector)));

        // Replace the focus handler to ensure it is registered after the FAB component is registered
        this._initCommands();
        this._initFocusHandler();
    }

    private _initCommands(): void {
        [
            KeyboardSetModeOperation,
            KeyboardInsertTextOperation,
            KeyboardDeleteBackwardOperation,
            KeyboardConfirmAndMoveOperation,
            KeyboardInsertFunctionOperation,
            KeyboardInsertQuotesOperation,
            KeyboardShowOperation,
            KeyboardHideOperation,
        ].forEach((command) => this.disposeWithMe(this._commandService.registerCommand(command)));
    }

    private _initFocusHandler(): void {
        this.disposeWithMe(
            this._layoutService.registerFocusHandler(UniverInstanceType.UNIVER_SHEET, (_unitId: string) => {
                const mobileKeyboardService = this._mobileKeyboardService;
                const renderManagerService = this._injector.get(IRenderManagerService);
                const instanceService = this._injector.get(IUniverInstanceService);
                const currentEditorRender = getCurrentTypeOfRenderer(UniverInstanceType.UNIVER_DOC, instanceService, renderManagerService);
                const docSelectionRenderService = currentEditorRender?.with(DocSelectionRenderService);

                docSelectionRenderService?.setInputMode('');
                this.disposeWithMe(mobileKeyboardService.keyboardMode$.subscribe((mode) => {
                    if (mode === 'text') {
                        docSelectionRenderService?.setInputMode('');
                        docSelectionRenderService?.focus();
                    } else {
                        docSelectionRenderService?.setInputMode('none');
                    }
                }));
            }, true)
        );
    }
}
