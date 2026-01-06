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

import { Disposable, EDITOR_ACTIVATED, ICommandService, IContextService, Inject, Injector } from '@univerjs/core';
import { IFindReplaceService } from '@univerjs/find-replace';
import { filter } from 'rxjs';
import { SheetReplaceCommand } from '../../commands/commands/sheet-replace.command';
import { SheetsFindReplaceProvider } from '../sheet-find-replace.controller';

/**
 * Mobile controller for sheets find-replace feature.
 * Uses mobile sidebar controller instead of desktop dialog controller.
 */
export class SheetsFindReplaceMobileController extends Disposable {
    private _provider!: SheetsFindReplaceProvider;

    constructor(
        @Inject(Injector) private readonly _injector: Injector,
        @IContextService private readonly _contextService: IContextService,
        @IFindReplaceService private readonly _findReplaceService: IFindReplaceService,
        @ICommandService private readonly _commandService: ICommandService
    ) {
        super();

        this._init();
        this._initCommands();
    }

    override dispose(): void {
        super.dispose();

        // Terminate find-replace session (mobile sidebar controller will handle closing the sidebar)
        this._findReplaceService.terminate();
        this._provider.dispose();
    }

    private _init(): void {
        const provider = this._injector.createInstance(SheetsFindReplaceProvider);
        this._provider = provider;

        this.disposeWithMe(this._findReplaceService.registerFindReplaceProvider(provider));

        // The find replace sidebar should be closed when sheet cell editor is activated, or the formula editor is focused.
        this.disposeWithMe(this._contextService.subscribeContextValue$(EDITOR_ACTIVATED)
            .pipe(filter((v) => !!v))
            .subscribe(() => {
                // Terminate find-replace session (mobile sidebar controller will handle closing the sidebar)
                this._findReplaceService.terminate();
            }));
    }

    private _initCommands(): void {
        [SheetReplaceCommand].forEach((command) => this.disposeWithMe(this._commandService.registerCommand(command)));
    }
}
