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

import { Disposable, ICommandService, Inject, Injector } from '@univerjs/core';
import { BuiltInUIPart, connectInjector, IUIPartsService } from '@univerjs/ui';
import { IEditorBridgeService } from '../../services/editor-bridge.service';
import { IMobileKeyboardService } from '../../services/mobile/mobile-keyboard.service';
import { KeyboardFab } from '../../views/mobile/fab/KeyboardFab';
import { KeyboardContainer } from '../../views/mobile/keyboard/common/KeyboardContainer';

export class MobileKeyboardController extends Disposable {
    constructor(
        @Inject(IMobileKeyboardService) private readonly _mobileKeyboardService: IMobileKeyboardService,
        @Inject(IEditorBridgeService) private readonly _editorBridgeService: IEditorBridgeService,
        @Inject(ICommandService) private readonly _commandService: ICommandService,
        @Inject(Injector) private readonly _injector: Injector,
        @Inject(IUIPartsService) private readonly _uiPartsService: IUIPartsService
    ) {
        super();

        // Register the FAB component to be rendered
        this.disposeWithMe(this._uiPartsService.registerComponent(BuiltInUIPart.GLOBAL, () => connectInjector(KeyboardFab, this._injector)));

        // Register the Keyboard Container component to be rendered
        this.disposeWithMe(this._uiPartsService.registerComponent(BuiltInUIPart.FOOTER, () => connectInjector(KeyboardContainer, this._injector)));

        // Initialize subscriptions for cell focus and keyboard coordination
        this._initCellFocusMonitoring();
    }

    private _initCellFocusMonitoring(): void {
        // Subscribe to cell focus events to trigger FAB display logic
        // The FAB component will subscribe to isKeyboardVisible$ to know when to hide
        // This controller coordinates between the editor state and keyboard UI state

        this.disposeWithMe(
            this._editorBridgeService.currentEditCellState$.subscribe((editState) => {
                // When a cell enters edit state, ensure keyboard UI is visible
                // if (editState && !this._editorBridgeService.isVisible().visible) {
                //     // Editor is about to become visible, keyboard UI will sync via service subscription
                //     this._mobileKeyboardService.showKeyboard();
                // } else {
                //     this._mobileKeyboardService.hideKeyboard();
                // }
            })
        );
    }
}
