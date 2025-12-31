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
import { DependentOn, Inject, Injector, mergeOverrideWithDependencies, Plugin, registerDependencies, touchDependencies, UniverInstanceType } from '@univerjs/core';
import { UniverSheetsFormulaUIPlugin } from '@univerjs/sheets-formula-ui';
import { UniverSheetsUIPlugin } from '@univerjs/sheets-ui';
import { MobileKeyboardController } from './controllers/mobile-keyboard.controller';
import { MobileViewportController } from './controllers/mobile-viewport.controller';
import { IMobileKeyboardService, MobileKeyboardService } from './services/mobile-keyboard.service';

@DependentOn(UniverSheetsUIPlugin, UniverSheetsFormulaUIPlugin)
export class UniverMobileKeyboardUIPlugin extends Plugin {
    static override pluginName = 'MOBILE_KEYBOARD_UI_PLUGIN';
    static override type = UniverInstanceType.UNIVER_SHEET;

    constructor(
        @Inject(Injector) protected readonly _injector: Injector
    ) {
        super();
    }

    override onStarting(): void {
        registerDependencies(this._injector, mergeOverrideWithDependencies([
            [IMobileKeyboardService, { useClass: MobileKeyboardService }],
            [MobileKeyboardController],
            [MobileViewportController],
        ] as Dependency[], undefined));
    }

    override onReady(): void {
        touchDependencies(this._injector, [
            [MobileKeyboardController],
            [MobileViewportController],
        ]);
    }
}
