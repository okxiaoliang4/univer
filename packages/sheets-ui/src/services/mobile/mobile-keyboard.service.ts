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

import type { Observable } from 'rxjs';
import type { IEditorBridgeService } from '../editor-bridge.service';
import { createIdentifier, Disposable, Inject } from '@univerjs/core';
import { BehaviorSubject } from 'rxjs';
import { IEditorBridgeService as IEditorBridgeServiceToken } from '../editor-bridge.service';

export type KeyboardMode = 'formula' | 'number' | 'text';

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
}

export const IMobileKeyboardService = createIdentifier<IMobileKeyboardService>('sheets-ui.mobile-keyboard.service');

export class MobileKeyboardService extends Disposable implements IMobileKeyboardService {
    private readonly _isKeyboardVisible$ = new BehaviorSubject<boolean>(false);
    readonly isKeyboardVisible$ = this._isKeyboardVisible$.asObservable();

    private readonly _keyboardMode$ = new BehaviorSubject<KeyboardMode>('number');
    readonly keyboardMode$ = this._keyboardMode$.asObservable();

    private _lastUsedMode: KeyboardMode = 'number';

    constructor(
        @Inject(IEditorBridgeServiceToken) private readonly _editorBridgeService: IEditorBridgeService
    ) {
        super();

        // Sync keyboard UI visibility with editor visibility
        // When editor becomes visible, show keyboard UI; when editor hides, hide keyboard UI
        this.disposeWithMe(
            this._editorBridgeService.visible$.subscribe(({ visible }) => {
                if (visible && !this._isKeyboardVisible$.value) {
                    this._autoSelectMode();
                    this._isKeyboardVisible$.next(true);
                }
            })
        );
    }

    private _autoSelectMode(): void {
        const editState = this._editorBridgeService.getEditCellState();
        if (!editState) {
            this.setMode(this._lastUsedMode);
            return;
        }

        const { documentLayoutObject } = editState;
        const cellValue = documentLayoutObject.documentModel?.getBody()?.dataStream ?? '';

        // If it starts with =, it's a formula
        if (cellValue.startsWith('=')) {
            this.setMode('formula');
            return;
        }

        // Check cell type if available
        // Note: In Univer, cell.t is often used for type.
        // We can check if it's numeric.
        const isNumeric = !Number.isNaN(Number(cellValue.trim())) && cellValue.trim() !== '';
        if (isNumeric) {
            this.setMode('number');
        } else {
            // Default to text if it's not a formula or number
            this.setMode('text');
        }
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
}
