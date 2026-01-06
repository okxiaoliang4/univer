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

import type { IDisposable, Nullable } from '@univerjs/core';
import {
    ICommandService,
    Inject,
    IUniverInstanceService,
    LocaleService,
    RxDisposable,
    toDisposable,
} from '@univerjs/core';
import { SearchIcon } from '@univerjs/icons';
import { ComponentManager, ILayoutService, IMenuManagerService, ISidebarService } from '@univerjs/ui';
import { takeUntil } from 'rxjs';
import { ReplaceAllMatchesCommand, ReplaceCurrentMatchCommand } from '../../commands/commands/replace.command';
import {
    FocusSelectionOperation,
    GoToNextMatchOperation,
    GoToPreviousMatchOperation,
    OpenFindDialogOperation,
    OpenReplaceDialogOperation,
} from '../../commands/operations/find-replace.operation';
import { IFindReplaceService } from '../../services/find-replace.service';
import { MobileFindReplace } from '../../views/mobile/MobileFindReplace';
import { menuSchema } from '../menu.schema';

const FIND_REPLACE_SIDEBAR_ID = 'MOBILE_FIND_REPLACE_SIDEBAR';

/**
 * Mobile controller for find-replace feature.
 * Similar to FindReplaceController but uses Sidebar instead of Dialog.
 * Registers commands but not shortcuts (mobile devices don't have keyboards).
 */
export class MobileFindReplaceController extends RxDisposable {
    constructor(
        @IUniverInstanceService private readonly _univerInstanceService: IUniverInstanceService,
        @IMenuManagerService private readonly _menuManagerService: IMenuManagerService,
        @ICommandService private readonly _commandService: ICommandService,
        @IFindReplaceService private readonly _findReplaceService: IFindReplaceService,
        @ISidebarService private readonly _sidebarService: ISidebarService,
        @ILayoutService private readonly _layoutService: ILayoutService,
        @Inject(LocaleService) private readonly _localeService: LocaleService,
        @Inject(ComponentManager) private readonly _componentManager: ComponentManager
    ) {
        super();

        this._initCommands();
        this._initUI();
        // Note: No shortcuts registration for mobile (no keyboard)
    }

    override dispose(): void {
        super.dispose();

        this._closingListenerDisposable?.dispose();
        this._closingListenerDisposable = null;
    }

    private _initCommands(): void {
        [
            OpenFindDialogOperation,
            OpenReplaceDialogOperation,
            GoToNextMatchOperation,
            GoToPreviousMatchOperation,
            ReplaceAllMatchesCommand,
            ReplaceCurrentMatchCommand,
            FocusSelectionOperation,
        ].forEach((c) => {
            this.disposeWithMe(this._commandService.registerCommand(c));
        });
    }

    private _initUI(): void {
        ([
            [MobileFindReplace.componentKey, MobileFindReplace],
            ['SearchIcon', SearchIcon],
        ] as const).forEach(([key, comp]) => {
            this.disposeWithMe(
                this._componentManager.register(key, comp)
            );
        });

        this._menuManagerService.mergeMenu(menuSchema);

        // This controller is responsible for toggling the find-replace sidebar
        this._findReplaceService.stateUpdates$.pipe(takeUntil(this.dispose$)).subscribe((newState) => {
            if (newState.revealed === true) {
                this._openSidebar();
            } else if (newState.revealed === false) {
                this._closeSidebar();
            }
        });
    }

    private _openSidebar(): void {
        // Close existing sidebar if any
        if (this._sidebarDisposable) {
            this._closeSidebar();
        }

        const title = this._localeService.t('find-replace.dialog.title');

        this._sidebarDisposable = this._sidebarService.open({
            id: FIND_REPLACE_SIDEBAR_ID,
            header: {
                title,
            },
            children: {
                label: MobileFindReplace.componentKey,
            },
            width: '100%',
            onClose: () => this.closeSidebar(),
        });

        this._closingListenerDisposable = toDisposable(this._univerInstanceService.focused$.pipe(takeUntil(this.dispose$)).subscribe((focused) => {
            if (!focused || !this._univerInstanceService.getUniverSheetInstance(focused)) {
                this.closeSidebar();
            }
        }));
    }

    private _sidebarDisposable: Nullable<IDisposable>;
    private _closingListenerDisposable: Nullable<IDisposable>;

    closeSidebar(): void {
        if (!this._closingListenerDisposable) {
            return;
        }

        this._closingListenerDisposable.dispose();
        this._closingListenerDisposable = null;

        this._closeSidebar();
        this._findReplaceService.terminate();

        queueMicrotask(() => this._layoutService.focus());
    }

    private _closeSidebar(): void {
        if (this._sidebarDisposable) {
            this._sidebarDisposable.dispose();
            this._sidebarDisposable = null;
        }
        this._sidebarService.close(FIND_REPLACE_SIDEBAR_ID);
    }
}
