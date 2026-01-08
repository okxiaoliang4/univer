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
import { Disposable, Inject, LocaleService } from '@univerjs/core';
import { ISidebarService } from '@univerjs/ui';
import { ISheetsHyperLinkPopupService } from '../../services/popup.service';
import { MobileCellLinkEdit } from '../../views/mobile/MobileCellLinkEdit';

const SIDEBAR_ID = 'sheets-hyper-link-edit';

export class SheetsHyperLinkMobilePopupController extends Disposable {
    private _sidebarDisposable: IDisposable | null = null;

    constructor(
        @Inject(ISheetsHyperLinkPopupService) private readonly _popupService: ISheetsHyperLinkPopupService,
        @Inject(ISidebarService) private readonly _sidebarService: ISidebarService,
        @Inject(LocaleService) private readonly _localeService: LocaleService
    ) {
        super();

        this._initSidebarListener();
    }

    private _initSidebarListener() {
        this.disposeWithMe(
            this._popupService.currentEditing$.subscribe((editing) => {
                if (editing) {
                    this._openSidebar();
                } else {
                    this._closeSidebar();
                }
            })
        );

        // Also listen to popup service hide events
        this.disposeWithMe(
            this._popupService.currentPopup$.subscribe((popup) => {
                // If popup is hidden and there's no editing state, close sidebar
                if (!popup && !this._popupService.currentEditing) {
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

        const editing = this._popupService.currentEditing;
        if (!editing) {
            return;
        }

        // Determine if editing existing link or adding new one
        const isEditing = !!editing.customRangeId || !!editing.customRange;
        const title = isEditing
            ? this._localeService.t('hyperLink.form.editTitle')
            : this._localeService.t('hyperLink.form.addTitle');

        this._sidebarDisposable = this._sidebarService.open({
            id: SIDEBAR_ID,
            header: {
                title,
            },
            children: {
                label: MobileCellLinkEdit.componentKey,
            },
            width: '100%',
            onClose: () => {
                // When sidebar closes, end editing in popup service
                this._popupService.endEditing();
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
