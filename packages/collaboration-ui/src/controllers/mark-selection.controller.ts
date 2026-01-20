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

import type { IRenderModule } from '@univerjs/engine-render';
import { IAwarenessService } from '@univerjs/collaboration';
import { Disposable, Inject } from '@univerjs/core';
import { SheetSkeletonManagerService } from '@univerjs/sheets-ui';
import { IMarkSelectionService } from '../services/mark-selection.service';

export class MarkSelectionRenderController
    extends Disposable
    implements IRenderModule {
    constructor(
        _config: undefined,
        @Inject(IMarkSelectionService)
        private _markSelectionService: IMarkSelectionService,
        @Inject(SheetSkeletonManagerService)
        private _sheetSkeletonManagerService: SheetSkeletonManagerService,
        @IAwarenessService
        private _awarenessService: IAwarenessService
    ) {
        super();
        this._initListeners();
    }

    private _initListeners() {
        this._addRefreshListener();
    }

    private _addRefreshListener() {
        this.disposeWithMe(
            this._sheetSkeletonManagerService.currentSkeleton$.subscribe(
                (skeleton) => {
                    if (skeleton) {
                        this._markSelectionService.refreshShapes();
                    }
                }
            )
        );
    }
}
