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

import type { Workbook } from '@univerjs/core';
import type { IRenderContext, IRenderModule } from '@univerjs/engine-render';
import { Disposable, DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY, FOCUSING_SHEET, ICommandService, IContextService, Optional } from '@univerjs/core';
import { fromEvent } from 'rxjs';
import { ChangeZoomRatioCommand } from '../../../commands/commands/set-zoom-ratio.command';
import { IEditorBridgeService } from '../../../services/editor-bridge.service';
import { getSheetObject } from '../../utils/component-tools';

/**
 * Calculate the distance between two touch points
 */
function getDistance(touch1: Touch, touch2: Touch): number {
    const dx = touch2.clientX - touch1.clientX;
    const dy = touch2.clientY - touch1.clientY;
    return Math.sqrt(dx * dx + dy * dy);
}

/**
 * Mobile zoom render controller for pinch-to-zoom gesture
 */
export class MobileZoomRenderController extends Disposable implements IRenderModule {
    private _initialDistance: number = 0;
    private _initialZoomRatio: number = 1;
    private _isPinching: boolean = false;

    constructor(
        private readonly _context: IRenderContext<Workbook>,
        @ICommandService private readonly _commandService: ICommandService,
        @IContextService private readonly _contextService: IContextService,
        @Optional(IEditorBridgeService) private readonly _editorBridgeService?: IEditorBridgeService
    ) {
        super();

        this._initTouchEventListeners();
    }

    private _initTouchEventListeners(): void {
        const sheetObject = getSheetObject(this._context.unit, this._context);
        if (!sheetObject) return;

        const canvasElement = sheetObject.engine.getCanvasElement();
        if (!canvasElement) return;

        this.disposeWithMe(fromEvent(canvasElement, 'touchstart', { passive: false }).subscribe((e) => {
            this._handleTouchStart(e as TouchEvent);
        }));
        this.disposeWithMe(fromEvent(canvasElement, 'touchmove', { passive: false }).subscribe((e) => {
            this._handleTouchMove(e as TouchEvent);
        }));
        this.disposeWithMe(fromEvent(canvasElement, 'touchend').subscribe((e) => {
            this._handleTouchEnd(e as TouchEvent);
        }));
        this.disposeWithMe(fromEvent(canvasElement, 'touchcancel').subscribe(() => {
            this._handleTouchCancel();
        }));
    }

    private _handleTouchStart(e: TouchEvent): void {
        // Only handle when there are exactly 2 touches
        if (e.touches.length === 2) {
            // Check if editor is visible
            if (this._editorBridgeService) {
                const state = this._editorBridgeService.isVisible();
                if ((state.unitId === this._context.unitId || state.unitId === DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY) && state.visible) {
                    return;
                }
            }

            // Check if sheet is focused
            if (!this._contextService.getContextValue(FOCUSING_SHEET)) {
                return;
            }

            const touch1 = e.touches[0];
            const touch2 = e.touches[1];
            this._initialDistance = getDistance(touch1, touch2);

            const workbook = this._context.unit;
            const sheet = workbook.getActiveSheet();
            if (!sheet) return;

            this._initialZoomRatio = sheet.getZoomRatio() || 1;
            this._isPinching = true;

            // Prevent default scrolling behavior during pinch
            e.preventDefault();
        }
    }

    private _handleTouchMove(e: TouchEvent): void {
        if (!this._isPinching || e.touches.length !== 2) {
            return;
        }

        // Check if editor is visible
        if (this._editorBridgeService) {
            const state = this._editorBridgeService.isVisible();
            if ((state.unitId === this._context.unitId || state.unitId === DOCS_FORMULA_BAR_EDITOR_UNIT_ID_KEY) && state.visible) {
                this._resetPinchState();
                return;
            }
        }

        // Check if sheet is focused
        if (!this._contextService.getContextValue(FOCUSING_SHEET)) {
            this._resetPinchState();
            return;
        }

        const touch1 = e.touches[0];
        const touch2 = e.touches[1];
        const currentDistance = getDistance(touch1, touch2);

        if (this._initialDistance === 0) {
            return;
        }

        // Calculate zoom ratio change
        const distanceRatio = currentDistance / this._initialDistance;
        const newZoomRatio = this._initialZoomRatio * distanceRatio;

        // Clamp zoom ratio to valid range (0.1 to 4.0, which is 10% to 400%)
        const clampedZoomRatio = Math.max(0.1, Math.min(4.0, newZoomRatio));

        const workbook = this._context.unit;
        const sheet = workbook.getActiveSheet();
        if (!sheet) return;

        // Calculate delta for ChangeZoomRatioCommand (delta is ratio delta, not percentage)
        const currentRatio = sheet.getZoomRatio() || 1;
        const delta = clampedZoomRatio - currentRatio;

        // Only update if there's a meaningful change (at least 0.01 ratio change)
        if (Math.abs(delta) > 0.01) {
            this._commandService.executeCommand(ChangeZoomRatioCommand.id, {
                delta,
                reset: false,
            });
        }

        // Prevent default scrolling behavior
        e.preventDefault();
    }

    private _handleTouchEnd(e: TouchEvent): void {
        // Reset when touches drop below 2
        // Note: e.touches contains remaining touches, e.changedTouches contains removed touches
        if (e.touches.length < 2) {
            this._resetPinchState();
        }
    }

    private _handleTouchCancel(): void {
        this._resetPinchState();
    }

    private _resetPinchState(): void {
        this._isPinching = false;
        this._initialDistance = 0;
        this._initialZoomRatio = 1;
    }
}
