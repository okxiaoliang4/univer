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
import type { ISheetLocationBase } from '@univerjs/sheets';
import type { Observable } from 'rxjs';
import { createIdentifier, Disposable, DisposableCollection, Inject } from '@univerjs/core';
import { CellPopupManagerService, SheetCanvasPopManagerService } from '@univerjs/sheets-ui';
import { IZenZoneService } from '@univerjs/ui';
import { BehaviorSubject } from 'rxjs';
import { SHEETS_THREAD_COMMENT_MODAL } from '../types/const';

export interface IThreadCommentPopup extends ISheetLocationBase {
    commentId?: string;
    // when triggered by hover, temp is set to be `true`
    temp?: boolean;
    trigger?: string;
}

/**
 * Interface for thread comment popup/sidebar service.
 * Platform-specific implementations handle popup display:
 * - Desktop: Uses canvas popup via `CellPopupManagerService`
 * - Mobile: Uses sidebar via `ISidebarService`
 *
 * Controllers and components inject this interface, allowing them to work
 * with both desktop and mobile implementations without platform-specific code.
 */
export interface ISheetsThreadCommentPopupService {
    /** Observable stream of active popup state. Emits `null` when no popup is active. */
    activePopup$: Observable<Nullable<IThreadCommentPopup>>;

    /** Get current active popup state. Returns `null` if no popup is active. */
    get activePopup(): Nullable<IThreadCommentPopup>;

    /**
     * Show popup/sidebar with comment interface.
     * @param location - Popup location and metadata (unitId, subUnitId, row, col, commentId, etc.)
     * @param onHide - Optional callback invoked when popup is hidden
     */
    showPopup(location: IThreadCommentPopup, onHide?: () => void): void;

    /**
     * Hide popup/sidebar and clear active state.
     */
    hidePopup(): void;

    /**
     * Make temporary popup persistent.
     * Converts a temporary popup (temp: true) to persistent (temp: false).
     * No-op if popup is already persistent or no popup is active.
     */
    persistPopup(): void;
}

/**
 * Redi dependency injection token for thread comment popup service.
 */
export const ISheetsThreadCommentPopupService = createIdentifier<ISheetsThreadCommentPopupService>(
    'sheets-thread-comment-ui.sheets-thread-comment-popup.service'
);

/**
 * Desktop implementation of thread comment popup service.
 * Uses canvas popups via `CellPopupManagerService` to display comment interface
 * as an overlay on the spreadsheet canvas.
 *
 * Registered in `UniverSheetsThreadCommentUIPlugin` via dependency injection.
 */
export class SheetsThreadCommentDesktopPopupService extends Disposable implements ISheetsThreadCommentPopupService {
    private _lastPopup: Nullable<IDisposable> = null;
    private _activePopup: Nullable<IThreadCommentPopup>;
    private _activePopup$ = new BehaviorSubject<Nullable<IThreadCommentPopup>>(null);

    activePopup$ = this._activePopup$.asObservable();

    get activePopup() {
        return this._activePopup;
    }

    constructor(
        @Inject(SheetCanvasPopManagerService) private readonly _canvasPopupManagerService: SheetCanvasPopManagerService,
        @IZenZoneService private readonly _zenZoneService: IZenZoneService,
        @Inject(CellPopupManagerService) private readonly _cellPopupManagerService: CellPopupManagerService
    ) {
        super();
        this._initZenVisible();

        this.disposeWithMe(() => {
            this._activePopup$.complete();
        });
    }

    private _initZenVisible() {
        this.disposeWithMe(this._zenZoneService.visible$.subscribe((visible) => {
            if (visible) {
                this.hidePopup();
            }
        }));
    }

    override dispose(): void {
        super.dispose();
        this.hidePopup();
    }

    showPopup(location: IThreadCommentPopup, onHide?: () => void) {
        const { row, col, unitId, subUnitId } = location;
        if (
            this._activePopup &&
            row === this._activePopup.row &&
            col === this._activePopup.col &&
            unitId === this._activePopup.unitId &&
            subUnitId === this.activePopup?.subUnitId
        ) {
            this._activePopup = location;
            this._activePopup$.next(location);
            return;
        }
        if (this._lastPopup) {
            this._lastPopup.dispose();
        };
        if (this._zenZoneService.visible) {
            return;
        }

        this._activePopup = location;
        this._activePopup$.next(location);

        const popupDisposable = this._cellPopupManagerService.showPopup(
            {
                row,
                col,
                unitId,
                subUnitId,
            },
            {
                componentKey: SHEETS_THREAD_COMMENT_MODAL,
                onClickOutside: () => {
                    this.hidePopup();
                },
                direction: 'horizontal',
                excludeOutside: [
                    ...Array.from(document.querySelectorAll('.univer-thread-comment')),
                    document.getElementById('thread-comment-add'),
                ].filter(Boolean) as HTMLElement[],
                priority: 2,
            }
        );

        if (!popupDisposable) {
            throw new Error('[SheetsThreadCommentDesktopPopupService]: cannot show popup!');
        }

        const disposableCollection = new DisposableCollection();
        disposableCollection.add(popupDisposable);
        disposableCollection.add({
            dispose: () => {
                onHide?.();
            },
        });

        this._lastPopup = disposableCollection;
    }

    hidePopup() {
        if (!this._activePopup) {
            return;
        }
        if (this._lastPopup) {
            this._lastPopup.dispose();
        }
        this._lastPopup = null;

        this._activePopup = null;
        this._activePopup$.next(null);
    }

    persistPopup() {
        if (!this._activePopup || !this._activePopup.temp) {
            return;
        }
        this._activePopup = {
            ...this._activePopup,
            temp: false,
        };

        this._activePopup$.next(this._activePopup);
    }
}
