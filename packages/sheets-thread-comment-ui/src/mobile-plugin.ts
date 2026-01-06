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

import type { Dependency } from '@univerjs/core';
import { DependentOn, Inject, Injector, Plugin, UniverInstanceType } from '@univerjs/core';
import { UniverMobileUIPlugin } from '@univerjs/ui';
import { SheetsThreadCommentMobileHoverController } from './controllers/mobile/mobile-hover.controller';
import { SheetsThreadCommentMobilePopupController } from './controllers/mobile/mobile-popup.controller';
import { UniverSheetsThreadCommentUIPlugin } from './plugin';

@DependentOn(UniverSheetsThreadCommentUIPlugin, UniverMobileUIPlugin)
export class UniverSheetsThreadCommentMobileUIPlugin extends Plugin {
    static override pluginName = 'SHEET_THREAD_COMMENT_MOBILE_UI_PLUGIN';
    static override type = UniverInstanceType.UNIVER_SHEET;

    constructor(
        @Inject(Injector) protected override _injector: Injector
    ) {
        super();
    }

    override onStarting(): void {
        const dependencies: Dependency[] = [
            [SheetsThreadCommentMobilePopupController],
            [SheetsThreadCommentMobileHoverController],
        ];
        dependencies.forEach((dep) => this._injector.add(dep));
    }

    override onRendered(): void {
        // Initialize the controllers after rendering to ensure services are available
        this._injector.get(SheetsThreadCommentMobilePopupController);
        this._injector.get(SheetsThreadCommentMobileHoverController);
    }
}
