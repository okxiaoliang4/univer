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

import type { IFreeze, IRange, IWorksheetData, Nullable, Workbook } from '@univerjs/core';
import type { IMouseEvent, IPoint, IPointerEvent, IRenderContext, IRenderModule, IScrollObserverParam } from '@univerjs/engine-render';
import type { IScrollToCellOperationParams } from '@univerjs/sheets';
import type { IExpandSelectionCommandParams } from '../../../commands/commands/set-selection.command';
import type { IScrollState, IScrollStateSearchParam, IViewportScrollState } from '../../../services/scroll-manager.service';

import type { ISheetSkeletonManagerParam } from '../../../services/sheet-skeleton-manager.service';
import {
    Direction,
    Disposable,
    ICommandService,
    Inject,
    IUniverInstanceService,
    RANGE_TYPE,
    toDisposable,
} from '@univerjs/core';
import { IRenderManagerService, SHEET_VIEWPORT_KEY } from '@univerjs/engine-render';
import { ScrollToCellOperation, SheetsSelectionsService } from '@univerjs/sheets';
import { ScrollCommand, SetScrollRelativeCommand } from '../../../commands/commands/set-scroll.command';
import { ExpandSelectionCommand, MoveSelectionCommand, MoveSelectionEnterAndTabCommand } from '../../../commands/commands/set-selection.command';
import { SheetScrollManagerService } from '../../../services/scroll-manager.service';
import { SheetSkeletonManagerService } from '../../../services/sheet-skeleton-manager.service';
import { getSheetObject } from '../../utils/component-tools';

const SHEET_NAVIGATION_COMMANDS = [MoveSelectionCommand.id, MoveSelectionEnterAndTabCommand.id];

/**
 * Touch history data structure for calculating velocity
 */
interface ITouchHistory {
    x: number;
    y: number;
    timestamp: number;
}

/**
 * Configuration constants for momentum scrolling
 */
const MAX_HISTORY_SIZE = 10; // Maximum number of touch history frames to keep
const DECELERATION = 0.96; // Per-frame deceleration factor (0.96 = 4% slower each frame)
const MIN_VELOCITY_PX = 0.01; // Minimum velocity in pixels to continue animation
const VELOCITY_MULTIPLIER = 20; // Multiplier to convert px/ms to px/frame (approx 60fps)

/**
 * This controller handles scroll logic in sheet interaction.
 */
export class MobileSheetsScrollRenderController extends Disposable implements IRenderModule {
    constructor(
        private readonly _context: IRenderContext<Workbook>,
        @Inject(SheetSkeletonManagerService) private readonly _sheetSkeletonManagerService: SheetSkeletonManagerService,
        @ICommandService private readonly _commandService: ICommandService,
        @IRenderManagerService private readonly _renderManagerService: IRenderManagerService,
        @Inject(SheetsSelectionsService) private readonly _selectionManagerService: SheetsSelectionsService,
        @Inject(SheetScrollManagerService) private readonly _scrollManagerService: SheetScrollManagerService,
        @IUniverInstanceService protected readonly _univerInstanceService: IUniverInstanceService
    ) {
        super();

        this._init();
    }

    scrollToRange(range: IRange): boolean {
        let { endRow, endColumn, startColumn, startRow } = range;
        const bounding = this._getViewportBounding();
        if (range.rangeType === RANGE_TYPE.ROW) {
            startColumn = 0;
            endColumn = 0;
        } else if (range.rangeType === RANGE_TYPE.COLUMN) {
            startRow = 0;
            endRow = 0;
        }

        if (bounding) {
            const row = bounding.startRow > endRow ? startRow : endRow;
            const col = bounding.startColumn > endColumn ? startColumn : endColumn;
            return this._scrollToCell(row, col);
        } else {
            return this._scrollToCell(startRow, startColumn);
        }
    }

    private _init() {
        this._initCommandListener();
        this._initScrollEventListener();
        this._initPointerScrollEvent();
        this._initSkeletonListener();
    }

    private _initCommandListener(): void {
        this.disposeWithMe(
            this._commandService.onCommandExecuted((command) => {
                if (SHEET_NAVIGATION_COMMANDS.includes(command.id)) {
                    this._scrollToSelection();
                } else if (command.id === ScrollToCellOperation.id) {
                    const param = (command.params as IScrollToCellOperationParams).range;
                    this.scrollToRange(param);
                } else if (command.id === ExpandSelectionCommand.id) {
                    const param = command.params as IExpandSelectionCommandParams;
                    this._scrollToSelectionForExpand(param);
                }
            })
        );
    }

    private _scrollToSelectionForExpand(param: IExpandSelectionCommandParams) {
        setTimeout(() => {
            const selection = this._selectionManagerService.getCurrentLastSelection();
            if (selection == null) {
                return;
            }

            const { startRow, startColumn, endRow, endColumn } = selection.range;

            const bounds = this._getViewportBounding();
            if (bounds == null) {
                return;
            }

            const { startRow: viewportStartRow, startColumn: viewportStartColumn, endRow: viewportEndRow, endColumn: viewportEndColumn } = bounds;

            let row = 0;
            let column = 0;

            if (startRow > viewportStartRow) {
                row = endRow;
            } else if (endRow < viewportEndRow) {
                row = startRow;
            } else {
                row = viewportStartRow;
            }

            if (startColumn > viewportStartColumn) {
                column = endColumn;
            } else if (endColumn < viewportEndColumn) {
                column = startColumn;
            } else {
                column = viewportStartColumn;
            }

            if (param.direction === Direction.DOWN) {
                row = endRow;
            } else if (param.direction === Direction.UP) {
                row = startRow;
            } else if (param.direction === Direction.RIGHT) {
                column = endColumn;
            } else if (param.direction === Direction.LEFT) {
                column = startColumn;
            }

            this._scrollToCell(row, column);
        }, 0);
    }

    private _getFreeze(): Nullable<IFreeze> {
        const snapshot: IWorksheetData | undefined = this._sheetSkeletonManagerService.getCurrentParam()?.skeleton.getWorksheetConfig();
        if (snapshot == null) {
            return;
        }

        return snapshot.freeze;
    }

    // eslint-disable-next-line max-lines-per-function
    private _initScrollEventListener() {
        const { scene } = this._context;
        if (scene == null) return;

        const viewportMain = scene.getViewport(SHEET_VIEWPORT_KEY.VIEW_MAIN);
        if (!viewportMain) return;

        //#region scrollInfo$ subscriber ---> viewport.scrollTo
        this.disposeWithMe(
            toDisposable(
                // wheel event --> set-scroll.command('sheet.operation.set-scroll') --> scroll.operation.ts -->
                // scrollManagerService.setScrollInfoAndEmitEvent --->  scrollManagerService.setScrollInfo(raw value, may be negative) &&
                // _notifyCurrentScrollInfo
                this._scrollManagerService.rawScrollInfo$.subscribe((rawScrollInfo: Nullable<IScrollState>) => {
                    if (rawScrollInfo == null) {
                        viewportMain.scrollToViewportPos({
                            viewportScrollX: 0,
                            viewportScrollY: 0,
                        });
                        return;
                    }

                    const skeleton = this._sheetSkeletonManagerService.getCurrentParam()?.skeleton;
                    if (!skeleton) return;

                    // data source: scroll-manager.service.ts@_scrollInfo
                    const { sheetViewStartRow, sheetViewStartColumn, offsetX, offsetY } = rawScrollInfo;

                    const { startX, startY } = skeleton.getCellWithCoordByIndex(
                        sheetViewStartRow,
                        sheetViewStartColumn,
                        false
                    );

                    // viewportScrollXByEvent is not same as viewportScrollX, by event, the value may be negative, or over max
                    const viewportScrollX = startX + offsetX;
                    const viewportScrollY = startY + offsetY;

                    // const config = viewportMain.transViewportScroll2ScrollValue(viewportScrollXByEvent, viewportScrollYByEvent);
                    viewportMain.scrollToViewportPos({ viewportScrollX, viewportScrollY });
                })
            )
        );
        //#endregion

        //#region viewport.onScrollAfter$ --> setScrollInfoToCurrSheet & validViewportScrollInfo$
        this.disposeWithMe(
            viewportMain.onScrollAfter$.subscribeEvent((scrollAfterParam: IScrollObserverParam) => {
                if (!scrollAfterParam) return;
                const skeleton = this._sheetSkeletonManagerService.getCurrentParam()?.skeleton;
                if (skeleton == null) return;
                const sheetObject = this._getSheetObject();
                if (skeleton == null || sheetObject == null) return;

                const { viewportScrollX, viewportScrollY, scrollX, scrollY } = scrollAfterParam;

                // according to the actual scroll position, the most suitable row, column and offset combination is recalculated.
                const { row, column, rowOffset, columnOffset } = skeleton.getOffsetRelativeToRowCol(
                    viewportScrollX,
                    viewportScrollY
                );

                const scrollInfo: IViewportScrollState = {
                    sheetViewStartRow: row,
                    sheetViewStartColumn: column,
                    offsetX: columnOffset,
                    offsetY: rowOffset,
                    viewportScrollX,
                    viewportScrollY,
                    scrollX,
                    scrollY,
                };
                // lastestScrollInfo derived from viewportScrollX, viewportScrollY from onScrollAfter$
                this._scrollManagerService.setValidScrollStateToCurrSheet(scrollInfo);
                this._scrollManagerService.validViewportScrollInfo$.next({
                    ...scrollInfo,
                    scrollX,
                    scrollY,
                    viewportScrollX,
                    viewportScrollY,
                });
                // snapshot is diff by diff people!
                // this._scrollManagerService.setScrollInfoToSnapshot({ ...lastestScrollInfo, viewportScrollX, viewportScrollY });
            })
        );
        //#endregion

        //#region scroll by bar
        this.disposeWithMe(
            viewportMain.onScrollByBar$.subscribeEvent((param) => {
                const skeleton = this._sheetSkeletonManagerService.getCurrentParam()?.skeleton;
                if (skeleton == null || param.isTrigger === false) {
                    return;
                }

                const sheetObject = this._getSheetObject();
                if (skeleton == null || sheetObject == null) {
                    return;
                }
                const { viewportScrollX = 0, viewportScrollY = 0 } = param;

                const freeze = this._getFreeze();

                const { row, column, rowOffset, columnOffset } = skeleton.getOffsetRelativeToRowCol(
                    viewportScrollX,
                    viewportScrollY
                );

                this._commandService.executeCommand(ScrollCommand.id, {
                    sheetViewStartRow: row + (freeze?.ySplit || 0),
                    sheetViewStartColumn: column + (freeze?.xSplit || 0),
                    offsetX: columnOffset,
                    offsetY: rowOffset,
                });
            })
        );
        //#endregion
    }

    private _initSkeletonListener() {
        this.disposeWithMe(toDisposable(
            this._sheetSkeletonManagerService.currentSkeletonBefore$.subscribe((param) => {
                if (param == null) {
                    return;
                }
                const scrollParam = { unitId: param.unitId, sheetId: param.sheetId } as IScrollStateSearchParam;
                this._scrollManagerService.setSearchParam(scrollParam);
                const sheetObject = this._getSheetObject();
                if (!sheetObject) return;
                const scene = sheetObject.scene;
                const viewportMain = scene.getViewport(SHEET_VIEWPORT_KEY.VIEW_MAIN);
                const currScrollInfo = this._scrollManagerService.getScrollStateByParam(scrollParam);
                const { viewportScrollX, viewportScrollY } = this._scrollManagerService.calcViewportScrollFromRowColOffset(currScrollInfo as unknown as Nullable<IViewportScrollState>);
                if (viewportMain) {
                    if (currScrollInfo) {
                        viewportMain.viewportScrollX = viewportScrollX;
                        viewportMain.viewportScrollY = viewportScrollY;
                    } else {
                        viewportMain.viewportScrollX = 0;
                        viewportMain.viewportScrollY = 0;
                    }
                    this._updateSceneSize(param as unknown as ISheetSkeletonManagerParam);
                }
            })
        ));
    }

    /**
     * Calculate velocity from touch history using the last few points
     * @param history Array of touch history records
     * @returns Calculated velocity in px/ms
     */
    private _calculateVelocity(history: ITouchHistory[]): { x: number; y: number } {
        if (history.length < 2) {
            return { x: 0, y: 0 };
        }

        // Use the last few points (or all if less than 4) for more stable velocity
        const recentCount = Math.min(4, history.length);
        const startIdx = history.length - recentCount;
        const endIdx = history.length - 1;

        const first = history[startIdx];
        const last = history[endIdx];
        const dt = last.timestamp - first.timestamp;

        // Avoid division by zero and very small time intervals
        if (dt < 10) {
            // If time is too short, use last two points
            const prev = history[history.length - 2];
            const curr = history[history.length - 1];
            const smallDt = curr.timestamp - prev.timestamp;
            if (smallDt <= 0) {
                return { x: 0, y: 0 };
            }
            const result = {
                x: (curr.x - prev.x) / smallDt,
                y: (curr.y - prev.y) / smallDt,
            };
            return result;
        }

        const result = {
            x: (last.x - first.x) / dt,
            y: (last.y - first.y) / dt,
        };
        return result;
    }

    /**
     * for mobile
     */
    // eslint-disable-next-line max-lines-per-function
    private _initPointerScrollEvent() {
        const sheetObject = this._getSheetObject();
        if (!sheetObject) return;

        const scene = sheetObject.scene;
        const spreadsheet = sheetObject.spreadsheet;
        const viewportMain = scene.getViewport(SHEET_VIEWPORT_KEY.VIEW_MAIN);
        if (!viewportMain) return;

        const lastPointerPos: IPoint = { x: 0, y: 0 };
        let _pointerScrolling: boolean = false;
        const touchHistory: ITouchHistory[] = [];

        // Track active pointer IDs to detect multi-touch
        const activePointerIds = new Set<number>();
        let primaryPointerId: number | null = null; // The pointer ID we're tracking for scrolling

        // Current velocity for momentum scrolling (in pixels per frame)
        const velocity = { x: 0, y: 0 };
        let scrollInertiaAnimationID: null | number = null;
        let inertiaStartTime: number = 0; // Track when inertia started to prevent immediate cancellation

        /**
         * Simple per-frame deceleration momentum scrolling
         * Each frame, velocity is reduced by DECELERATION factor
         */
        const pointerScrollInertia = () => {
            // Check if velocity is below threshold - stop animation
            if (Math.abs(velocity.x) < MIN_VELOCITY_PX && Math.abs(velocity.y) < MIN_VELOCITY_PX) {
                scrollInertiaAnimationID = null;
                velocity.x = 0;
                velocity.y = 0;
                return;
            }

            // Apply scroll first (before deceleration)
            const offsetX = velocity.x;
            const offsetY = velocity.y;

            if (Math.abs(offsetX) > 0.1 || Math.abs(offsetY) > 0.1) {
                this._commandService.executeCommand(SetScrollRelativeCommand.id, { offsetX, offsetY });
                // Mark scene as dirty to trigger redraw
                scene.makeDirty(true);
            }

            // Apply deceleration for next frame
            velocity.x *= DECELERATION;
            velocity.y *= DECELERATION;

            // Continue animation
            scrollInertiaAnimationID = requestAnimationFrame(pointerScrollInertia);
        };

        const cancelInertiaAnimation = (force: boolean = false) => {
            // Prevent immediate cancellation after starting inertia (allow RAF callback to execute)
            // Give it at least 50ms grace period
            const timeSinceStart = performance.now() - inertiaStartTime;
            if (!force && timeSinceStart < 50 && scrollInertiaAnimationID !== null) {
                return;
            }
            if (scrollInertiaAnimationID !== null) {
                cancelAnimationFrame(scrollInertiaAnimationID);
                scrollInertiaAnimationID = null;
            }
            velocity.x = 0;
            velocity.y = 0;
        };

        spreadsheet.onPointerDown$.subscribeEvent((evt: IPointerEvent | IMouseEvent, state) => {
            const pointerId = (evt as IPointerEvent).pointerId ?? 0;

            // Add this pointer to active set
            activePointerIds.add(pointerId);

            // If there are multiple touches, cancel scrolling and don't start new one
            if (activePointerIds.size > 1) {
                cancelInertiaAnimation(true);
                _pointerScrolling = false;
                primaryPointerId = null;
                touchHistory.length = 0;
                return;
            }

            cancelInertiaAnimation(true); // Force cancel on new touch

            if (!viewportMain) return;

            // Only handle single touch
            primaryPointerId = pointerId;

            // Reset touch history
            touchHistory.length = 0;
            lastPointerPos.x = evt.offsetX;
            lastPointerPos.y = evt.offsetY;
            _pointerScrolling = true;

            // Record initial touch point
            touchHistory.push({
                x: evt.offsetX,
                y: evt.offsetY,
                timestamp: performance.now(),
            });

            state.stopPropagation();
        });

        spreadsheet.onPointerMove$.subscribeEvent((evt: IPointerEvent | IMouseEvent, state) => {
            const pointerId = (evt as IPointerEvent).pointerId ?? 0;

            // Only process move events for the primary pointer, and only if single touch
            if (!_pointerScrolling || activePointerIds.size > 1 || primaryPointerId !== pointerId) {
                return;
            }
            if (!viewportMain) return;

            const e = evt as IPointerEvent | IMouseEvent;
            const currentTime = performance.now();

            // Record touch position and timestamp
            touchHistory.push({
                x: e.offsetX,
                y: e.offsetY,
                timestamp: currentTime,
            });

            // Keep only recent history (last MAX_HISTORY_SIZE frames)
            if (touchHistory.length > MAX_HISTORY_SIZE) {
                touchHistory.shift();
            }

            // Calculate immediate delta for current frame scrolling
            const deltaX = -(e.offsetX - lastPointerPos.x);
            const deltaY = -(e.offsetY - lastPointerPos.y);

            // Apply immediate scroll during touch move
            if (deltaX !== 0 || deltaY !== 0) {
                this._commandService.executeCommand(SetScrollRelativeCommand.id, { offsetX: deltaX, offsetY: deltaY });
            }

            lastPointerPos.x = e.offsetX;
            lastPointerPos.y = e.offsetY;

            state.stopPropagation();
        });

        spreadsheet.onPointerUp$.subscribeEvent((evt: IPointerEvent | IMouseEvent) => {
            const pointerId = (evt as IPointerEvent).pointerId ?? 0;

            // Remove pointer from active set
            activePointerIds.delete(pointerId);

            // Only process if this was the primary pointer and we're still in single touch mode
            if (primaryPointerId !== pointerId || activePointerIds.size > 0) {
                // If there are still active touches, stop scrolling
                if (activePointerIds.size > 0) {
                    cancelInertiaAnimation(true);
                    _pointerScrolling = false;
                    primaryPointerId = null;
                    touchHistory.length = 0;
                }
                return;
            }

            _pointerScrolling = false;
            primaryPointerId = null;

            // Calculate final velocity from touch history
            if (touchHistory.length >= 2) {
                const calculatedVelocity = this._calculateVelocity(touchHistory);
                // Convert from px/ms to px/frame and reverse direction (scroll opposite to touch)
                velocity.x = -calculatedVelocity.x * VELOCITY_MULTIPLIER;
                velocity.y = -calculatedVelocity.y * VELOCITY_MULTIPLIER;

                // Only start inertia if velocity is significant
                if (Math.abs(velocity.x) > MIN_VELOCITY_PX || Math.abs(velocity.y) > MIN_VELOCITY_PX) {
                    inertiaStartTime = performance.now(); // Record when inertia starts
                    scrollInertiaAnimationID = requestAnimationFrame(() => {
                        pointerScrollInertia();
                    });
                }
            }

            // Clear touch history
            touchHistory.length = 0;
        });

        // trigger by scene.input-manager@_onPointerMove because currObject has changed
        spreadsheet.onPointerLeave$.subscribeEvent((evt: IPointerEvent | IMouseEvent) => {
            const pointerId = (evt as IPointerEvent).pointerId ?? 0;
            activePointerIds.delete(pointerId);

            if (primaryPointerId === pointerId) {
                _pointerScrolling = false;
                primaryPointerId = null;
            }
            cancelInertiaAnimation();
            touchHistory.length = 0;
        });
        spreadsheet.onPointerOut$.subscribeEvent((evt: IPointerEvent | IMouseEvent) => {
            const pointerId = (evt as IPointerEvent).pointerId ?? 0;
            activePointerIds.delete(pointerId);

            if (primaryPointerId === pointerId) {
                _pointerScrolling = false;
                primaryPointerId = null;
            }
            cancelInertiaAnimation();
            touchHistory.length = 0;
        });
        scene.onPointerOut$.subscribeEvent((evt: IPointerEvent | IMouseEvent) => {
            const pointerId = (evt as IPointerEvent).pointerId ?? 0;
            activePointerIds.delete(pointerId);

            if (primaryPointerId === pointerId) {
                _pointerScrolling = false;
                primaryPointerId = null;
            }
            cancelInertiaAnimation();
            touchHistory.length = 0;
        });
        scene.onPointerCancel$.subscribeEvent((evt: IPointerEvent | IMouseEvent) => {
            const pointerId = (evt as IPointerEvent).pointerId ?? 0;
            activePointerIds.delete(pointerId);

            if (primaryPointerId === pointerId) {
                _pointerScrolling = false;
                primaryPointerId = null;
            }
            cancelInertiaAnimation(true);
            touchHistory.length = 0;
        });
    }

    private _updateSceneSize(param: ISheetSkeletonManagerParam) {
        if (param == null) {
            return;
        }

        const { unitId } = this._context;
        const { skeleton } = param;
        const scene = this._renderManagerService.getRenderById(unitId)?.scene;

        if (skeleton == null || scene == null) {
            return;
        }

        const { rowTotalHeight, columnTotalWidth, rowHeaderWidthAndMarginLeft, columnHeaderHeightAndMarginTop } =
            skeleton;
        const workbook = this._context.unit;
        const worksheet = workbook.getActiveSheet();
        if (!worksheet) return;

        const zoomRatio = worksheet.getZoomRatio() || 1;
        scene?.setScaleValueOnly(zoomRatio, zoomRatio);
        scene?.transformByState({
            width: rowHeaderWidthAndMarginLeft + columnTotalWidth,
            height: columnHeaderHeightAndMarginTop + rowTotalHeight,
        });
    }

    private _getSheetObject() {
        return getSheetObject(this._context.unit, this._context);
    }

    private _scrollToSelectionByDirection(range: IRange) {
        const bounds = this._getViewportBounding();
        if (bounds == null) {
            return false;
        }
        const {
            startRow: viewportStartRow,
            startColumn: viewportStartColumn,
            endRow: viewportEndRow,
            endColumn: viewportEndColumn,
        } = bounds;

        let row = 0;
        let column = 0;

        const { startRow, startColumn, endRow, endColumn } = range;

        if (startRow >= viewportStartRow) {
            row = endRow;
        }

        if (endRow <= viewportEndRow) {
            row = startRow;
        }

        if (startColumn >= viewportStartColumn) {
            column = endColumn;
        }

        if (endColumn <= viewportEndColumn) {
            column = startColumn;
        }

        this._scrollToCell(row, column);
    }

    private _scrollToSelection(targetIsActualRowAndColumn = true) {
        const selection = this._selectionManagerService.getCurrentLastSelection();
        if (selection == null) {
            return;
        }

        const { startRow, startColumn, actualRow, actualColumn } = selection.primary;
        const selectionStartRow = targetIsActualRowAndColumn ? actualRow : startRow;
        const selectionStartColumn = targetIsActualRowAndColumn ? actualColumn : startColumn;

        this._scrollToCell(selectionStartRow, selectionStartColumn);
    }

    private _getViewportBounding() {
        const scene = this._getSheetObject()?.scene;
        if (scene == null) {
            return;
        }

        const viewport = scene.getViewport(SHEET_VIEWPORT_KEY.VIEW_MAIN);
        if (viewport == null) {
            return;
        }

        const skeleton = this._sheetSkeletonManagerService.getCurrentParam()?.skeleton;
        if (skeleton == null) {
            return;
        }

        return skeleton.getRangeByViewport(viewport.calcViewportInfo());
    }

    // eslint-disable-next-line max-lines-per-function, complexity
    private _scrollToCell(row: number, column: number): boolean {
        const { rowHeightAccumulation, columnWidthAccumulation } = this._sheetSkeletonManagerService.getCurrentParam()?.skeleton ?? {};

        if (rowHeightAccumulation == null || columnWidthAccumulation == null) return false;

        const scene = this._getSheetObject()?.scene;
        if (scene == null) return false;

        const viewport = scene.getViewport(SHEET_VIEWPORT_KEY.VIEW_MAIN);
        if (viewport == null) return false;

        const skeleton = this._sheetSkeletonManagerService.getCurrentParam()?.skeleton;
        if (skeleton == null) return false;

        const worksheet = this._context.unit.getActiveSheet();
        if (!worksheet) return false;

        const {
            startColumn: freezeStartColumn,
            startRow: freezeStartRow,
            ySplit: freezeYSplit,
            xSplit: freezeXSplit,
        } = worksheet.getFreeze();

        const bounds = this._getViewportBounding();
        if (bounds == null) return false;

        const {
            startRow: viewportStartRow,
            startColumn: viewportStartColumn,
            endRow: viewportEndRow,
            endColumn: viewportEndColumn,
        } = bounds;

        let startSheetViewRow: number | undefined;
        let startSheetViewColumn: number | undefined;

        // vertical overflow only happens when the selection's row is in not the freeze area
        if (row >= freezeStartRow && column >= freezeStartColumn - freezeXSplit) {
            // top overflow
            if (row <= viewportStartRow) {
                startSheetViewRow = row;
            }

            // bottom overflow
            if (row >= viewportEndRow) {
                const minRowAccumulation = rowHeightAccumulation[row] - viewport.height!;
                for (let r = viewportStartRow; r <= row; r++) {
                    if (rowHeightAccumulation[r] >= minRowAccumulation) {
                        startSheetViewRow = r + 1;
                        break;
                    }
                }
            }
        }
        // horizontal overflow only happens when the selection's column is in not the freeze area
        if (column >= freezeStartColumn && row >= freezeStartRow - freezeYSplit) {
            // left overflow
            if (column <= viewportStartColumn) {
                startSheetViewColumn = column;
            }

            // right overflow
            if (column >= viewportEndColumn) {
                const minColumnAccumulation = columnWidthAccumulation[column] - viewport.width!;
                for (let c = viewportStartColumn; c <= column; c++) {
                    if (columnWidthAccumulation[c] >= minColumnAccumulation) {
                        startSheetViewColumn = c + 1;
                        break;
                    }
                }
            }
        }

        if (startSheetViewRow === undefined && startSheetViewColumn === undefined) return false;

        const { offsetX, offsetY } = this._scrollManagerService.getCurrentScrollState() || {};
        return this._commandService.syncExecuteCommand(ScrollCommand.id, {
            sheetViewStartRow: startSheetViewRow,
            sheetViewStartColumn: startSheetViewColumn,
            offsetX: startSheetViewColumn === undefined ? offsetX : 0,
            offsetY: startSheetViewRow === undefined ? offsetY : 0,
        });
    }
}
