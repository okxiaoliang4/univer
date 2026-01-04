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

import { Disposable, DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY, DOCS_NORMAL_EDITOR_UNIT_ID_KEY, Inject, Injector, toDisposable } from '@univerjs/core';
import { IEditorService } from '@univerjs/docs-ui';
import { BehaviorSubject } from 'rxjs';

export class MobileViewportController extends Disposable {
    readonly offset$ = new BehaviorSubject<number>(0);

    constructor(
        @Inject(Injector) protected readonly _injector: Injector,
        @IEditorService private readonly _editorService: IEditorService
    ) {
        super();
        this._initVisualViewportListener();
    }

    private _handleVisualViewportResize(): void {
        if (!window.visualViewport) return;
        const offset = window.innerHeight - window.visualViewport.height;
        this.offset$.next(offset);
    }

    private _initVisualViewportListener(): void {
        if (!window.visualViewport) return;
        window.visualViewport.addEventListener('resize', this._handleVisualViewportResize.bind(this));
        this.disposeWithMe(toDisposable(() => {
            if (!window.visualViewport) return;
            window.visualViewport.removeEventListener('resize', this._handleVisualViewportResize.bind(this));
        }));

        this.disposeWithMe(this._editorService.editorAdd$.subscribe((editorUnitId) => {
            const editor = this._editorService.getEditor(editorUnitId);
            if (!editor) return;
            if (editor.getEditorId() !== DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY && editor.getEditorId() !== DOCS_NORMAL_EDITOR_UNIT_ID_KEY) return;
            const docSelectionRenderService = editor.docSelectionRenderService;
            if (!docSelectionRenderService) return;
            editor.disposeWithMe(docSelectionRenderService.onBlur$.subscribe(() => {
                this.offset$.next(0);
            }));
            editor.disposeWithMe(docSelectionRenderService.onFocus$.subscribe(() => {
                this._handleVisualViewportResize();
            }));
        }));
    }
}
