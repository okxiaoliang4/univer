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
import type { ISheetsThreadCommentPopupService, IThreadCommentPopup } from './sheets-thread-comment-popup.service';
import { Disposable, Inject, LocaleService, Tools } from '@univerjs/core';
import { ISidebarService } from '@univerjs/ui';
import { BehaviorSubject } from 'rxjs';
import { SHEETS_THREAD_COMMENT_MODAL } from '../types/const';

const SIDEBAR_ID = 'sheets-thread-comment-popup';

/**
 * Mobile implementation of thread comment popup service.
 * Uses sidebar via `ISidebarService` to display comment interface
 * as a slide-in panel from the right side of the screen.
 *
 * Registered in `UniverSheetsThreadCommentMobileUIPlugin` via dependency injection.
 */
export class SheetsThreadCommentMobilePopupService extends Disposable implements ISheetsThreadCommentPopupService {
    private _activePopup: Nullable<IThreadCommentPopup>;
    private _activePopup$ = new BehaviorSubject<Nullable<IThreadCommentPopup>>(null);
    private _sidebarDisposable: IDisposable | null = null;

    readonly activePopup$ = this._activePopup$.asObservable();

    get activePopup() {
        return this._activePopup;
    }

    constructor(
        @Inject(ISidebarService) private readonly _sidebarService: ISidebarService,
        @Inject(LocaleService) private readonly _localeService: LocaleService
    ) {
        super();

        this.disposeWithMe(() => {
            this._activePopup$.complete();
        });
    }

    override dispose(): void {
        this.hidePopup();
        super.dispose();
    }

    showPopup(location: IThreadCommentPopup, onHide?: () => void): void {
        const { row, col, unitId, subUnitId } = location;
        if (
            this._activePopup &&
            row === this._activePopup.row &&
            col === this._activePopup.col &&
            unitId === this._activePopup.unitId &&
            subUnitId === this._activePopup.subUnitId
        ) {
            this._activePopup = location;
            this._activePopup$.next(location);
            return;
        }

        // Close existing sidebar if any
        if (this._sidebarDisposable) {
            this._closeSidebar();
        }

        this._activePopup = location;
        this._activePopup$.next(location);

        // Generate sidebar title from cell reference
        const cellRef = `${Tools.chatAtABC(col)}${row + 1}`;
        const title = this._localeService.t('sheetThreadComment.menu.addComment') || `Comment ${cellRef}`;

        this._sidebarDisposable = this._sidebarService.open({
            id: SIDEBAR_ID,
            header: {
                title,
            },
            children: {
                label: SHEETS_THREAD_COMMENT_MODAL,
            },
            width: '100%',
            onClose: () => {
                // Clear sidebar disposable reference first to prevent re-entry
                this._sidebarDisposable = null;
                // When sidebar closes, hide popup in service (but _activePopup will be checked first)
                this.hidePopup();
                onHide?.();
            },
        });
    }

    hidePopup(): void {
        if (!this._activePopup) {
            return;
        }

        // Clear active popup state BEFORE closing sidebar to prevent onClose callback from causing infinite loop
        const wasActive = !!this._activePopup;
        this._activePopup = null;
        this._activePopup$.next(null);

        // Only close sidebar if it was actually active
        if (wasActive) {
            this._closeSidebar();
        }
    }

    persistPopup(): void {
        if (!this._activePopup || !this._activePopup.temp) {
            return;
        }
        this._activePopup = {
            ...this._activePopup,
            temp: false,
        };

        this._activePopup$.next(this._activePopup);
    }

    private _closeSidebar(): void {
        if (this._sidebarDisposable) {
            this._sidebarDisposable.dispose();
            this._sidebarDisposable = null;
        }
        this._sidebarService.close(SIDEBAR_ID);
    }
}
