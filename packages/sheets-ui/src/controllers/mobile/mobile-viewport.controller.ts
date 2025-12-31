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

import { Disposable, ILogService, Inject, Injector, IUniverInstanceService, toDisposable, UniverInstanceType } from '@univerjs/core';
import { DocSelectionRenderService } from '@univerjs/docs-ui';
import { getCurrentTypeOfRenderer, IRenderManagerService } from '@univerjs/engine-render';
import { BehaviorSubject } from 'rxjs';

export class MobileViewportController extends Disposable {
    readonly offset$ = new BehaviorSubject<number>(0);

    constructor(
        @Inject(Injector) protected readonly _injector: Injector,
        @ILogService private readonly _logService: ILogService
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

        const renderManagerService = this._injector.get(IRenderManagerService);
        const instanceService = this._injector.get(IUniverInstanceService);
        const currentEditorRender = getCurrentTypeOfRenderer(UniverInstanceType.UNIVER_DOC, instanceService, renderManagerService);
        const docSelectionRenderService = currentEditorRender?.with(DocSelectionRenderService);
        if (!docSelectionRenderService) {
            this._logService.warn('[MobileViewportController]', 'DocSelectionRenderService not found');
            return;
        }
        this.disposeWithMe(docSelectionRenderService.onBlur$.subscribe(() => {
            this.offset$.next(0);
        }));
        this.disposeWithMe(docSelectionRenderService.onFocus$.subscribe(() => {
            this._handleVisualViewportResize();
        }));
    }
}
