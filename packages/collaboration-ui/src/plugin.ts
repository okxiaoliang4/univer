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
import {

    Inject,
    Injector,
    Plugin,
    registerDependencies,
    touchDependencies,
    UniverInstanceType,
} from '@univerjs/core';
import { IRenderManagerService } from '@univerjs/engine-render';
import { CollaborationUIController } from './controllers/collaboration.ui.controller';
import { MarkSelectionRenderController } from './controllers/mark-selection.controller';
import { AwarenessUIService } from './services/awareness.ui.service';
import {
    IMarkSelectionService,
    MarkSelectionService,
} from './services/mark-selection.service';

export const COLLABORATION_UI_PLUGIN = 'COLLABORATION_UI_PLUGIN';
export class CollaborationUIPlugin extends Plugin {
    static override pluginName = COLLABORATION_UI_PLUGIN;

    constructor(
        private readonly _: any,
    // inject injector, required
        @Inject(Injector) override readonly _injector: Injector,
        @IRenderManagerService
        private readonly _renderManagerService: IRenderManagerService
    ) {
        super();
    }

    override onStarting() {
        const dependencies: Dependency[] = [
            [CollaborationUIController],
            [MarkSelectionRenderController],
            [IMarkSelectionService, { useClass: MarkSelectionService }],
            [AwarenessUIService],
        ];

        registerDependencies(this._injector, dependencies);

        touchDependencies(this._injector, [[CollaborationUIController]]);
    }

    override onReady(): void {
        touchDependencies(this._injector, [
            [MarkSelectionService],
            [AwarenessUIService],
        ]);
    }

    override onRendered() {
        this._registerRenderModules();
    }

    private _registerRenderModules(): void {
        const modules: Dependency[] = [[MarkSelectionRenderController]];
        modules.forEach((m) => {
            this.disposeWithMe(
                this._renderManagerService.registerRenderModule(
                    UniverInstanceType.UNIVER_SHEET,
                    m
                )
            );
        });
    }
}
