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

import type { IDisposable } from '@univerjs/core';
import { Disposable, Inject, LocaleService, Tools } from '@univerjs/core';
import { ISidebarService } from '@univerjs/ui';
import { SheetsThreadCommentPopupService } from '../../services/sheets-thread-comment-popup.service';
import { SHEETS_THREAD_COMMENT_MODAL } from '../../types/const';

const SIDEBAR_ID = 'sheets-thread-comment-popup';

export class SheetsThreadCommentMobilePopupController extends Disposable {
    private _sidebarDisposable: IDisposable | null = null;

    constructor(
        @Inject(SheetsThreadCommentPopupService) private readonly _popupService: SheetsThreadCommentPopupService,
        @Inject(ISidebarService) private readonly _sidebarService: ISidebarService,
        @Inject(LocaleService) private readonly _localeService: LocaleService
    ) {
        super();

        this._initSidebarListener();
    }

    private _initSidebarListener() {
        this.disposeWithMe(
            this._popupService.activePopup$.subscribe((popup) => {
                if (popup) {
                    this._openSidebar();
                } else {
                    this._closeSidebar();
                }
            })
        );
    }

    private _openSidebar() {
        // Close existing sidebar if any
        if (this._sidebarDisposable) {
            this._closeSidebar();
        }

        const activePopup = this._popupService.activePopup;
        if (!activePopup) {
            return;
        }

        // Use cell reference for title
        const { row, col } = activePopup;
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
                // When sidebar closes, hide popup in popup service
                this._popupService.hidePopup();
            },
        });
    }

    private _closeSidebar() {
        if (this._sidebarDisposable) {
            this._sidebarDisposable.dispose();
            this._sidebarDisposable = null;
        }
        this._sidebarService.close(SIDEBAR_ID);
    }

    override dispose(): void {
        this._closeSidebar();
        super.dispose();
    }
}
