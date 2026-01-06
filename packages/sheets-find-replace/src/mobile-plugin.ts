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
import { UniverFindReplaceMobilePlugin } from '@univerjs/find-replace';
import { UniverSheetsPlugin } from '@univerjs/sheets';
import { SheetsFindReplaceMobileController } from './controllers/mobile/sheet-find-replace.controller';

/**
 * Mobile plugin for sheets find-replace feature.
 * Registers mobile-specific controller that uses sidebar instead of dialog.
 */
@DependentOn(UniverFindReplaceMobilePlugin, UniverSheetsPlugin)
export class UniverSheetsFindReplaceMobilePlugin extends Plugin {
    static override pluginName = 'SHEET_FIND_REPLACE_MOBILE_PLUGIN';
    static override type = UniverInstanceType.UNIVER_SHEET;

    constructor(
        @Inject(Injector) protected override _injector: Injector
    ) {
        super();
    }

    override onStarting(): void {
        const dependencies: Dependency[] = [
            [SheetsFindReplaceMobileController],
        ];
        dependencies.forEach((dep) => this._injector.add(dep));
    }

    override onSteady(): void {
        // Initialize the controller after steady to ensure all dependencies are ready
        this._injector.get(SheetsFindReplaceMobileController);
    }
}
