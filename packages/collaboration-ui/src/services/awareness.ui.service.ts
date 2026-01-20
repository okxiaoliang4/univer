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

import type { ICommandInfo, IExecutionOptions } from '@univerjs/core';
import type {
    ISelectionWithStyle,
    ISetSelectionsOperationParams,
} from '@univerjs/sheets';
import type {
    IHoverRichTextPosition,
    SelectionControl,
} from '@univerjs/sheets-ui';
import { IAwarenessService } from '@univerjs/collaboration';
import {
    Disposable,
    DisposableCollection,
    getIntersectRange,
    ICommandService,
    Inject,
    Injector,
    LifecycleService,
    LifecycleStages,
    toDisposable,
    Tools,
} from '@univerjs/core';
import { IRenderManagerService } from '@univerjs/engine-render';
import { SetSelectionsOperation } from '@univerjs/sheets';
import {
    HoverManagerService,
    SheetCanvasPopManagerService,
    SheetSkeletonManagerService,
} from '@univerjs/sheets-ui';
import { throttle } from 'lodash-es';
import { filter } from 'rxjs';
import { CollaborationSelectionPopup } from '../components/collaboration-selection-popup';
import { IMarkSelectionService } from './mark-selection.service';

export const userColors = [
    '#30BCED',
    '#6EEB83',
    '#FFBC42',
    '#ECD444',
    '#EE6352',
    '#9AC2C9',
    '#8ACB88',
    '#1BE7FF',
];

const randomColor = () => {
    return userColors[Math.floor(Math.random() * userColors.length)];
};

interface IAwarenessState {
    clientID: number;
    id: string;
    name: string;
    selectionParams: ISetSelectionsOperationParams;
}

interface IMarkSelectionInfo {
    unitId: string;
    subUnitId: string;
    selection: ISelectionWithStyle;
    zIndex: number;
    control: SelectionControl | null;
    exits: string[];
}

export class AwarenessUIService extends Disposable {
    private _disposableCollection: DisposableCollection =
        new DisposableCollection();

    private _currentHoveringClientID: number | null = null;
    private _userColorMap: Map<string, Map<number, string>> = new Map();
    private _initUnitIds = new Set<string>();

    constructor(
        @Inject(Injector) readonly _injector: Injector,
        @ICommandService
        private readonly _commandService: ICommandService,
        @IMarkSelectionService
        private readonly _markSelectionService: IMarkSelectionService,
        @IRenderManagerService
        private readonly _renderManagerService: IRenderManagerService,
        @Inject(LifecycleService)
        private readonly _lifecycleService: LifecycleService,
        @IAwarenessService
        private readonly _awarenessService: IAwarenessService
    ) {
        super();

        this._initHooks();
        this._initAwarenessUI();
    }

    _initHooks() {
        this.disposeWithMe(
            this._lifecycleService.lifecycle$.subscribe((stage) => {
                if (stage === LifecycleStages.Ready) {
                    const hoverManagerService = this._injector.get(HoverManagerService);
                    this.disposeWithMe(
                        toDisposable(
                            hoverManagerService.currentRichText$
                                .pipe(filter((cell) => !!cell))
                                .subscribe(this._onCellHover.bind(this))
                        )
                    );
                }
            })
        );
    }

    disposePopup() {
        this._disposableCollection.dispose();
    }

    _onCellHover(cell: IHoverRichTextPosition) {
        this._disposableCollection.dispose();
        const shapeMap = this._markSelectionService.getShapeMap();
        Array.from(shapeMap.entries()).forEach(async ([k, v]) => {
            if (cell.unitId !== v.unitId) return;
            if (cell.subUnitId !== v.subUnitId) return;

            const intersectRange = getIntersectRange(v.selection.range, {
                startRow: cell.row,
                endRow: cell.row,
                startColumn: cell.col,
                endColumn: cell.col,
            });
            if (k.startsWith('collab-') && intersectRange) {
                const states = await this._awarenessService.getState(cell.unitId);
                if (!states) return;

                const { clientID, name } = states.get(
                    Number(k.split('-')[1])
                ) as IAwarenessState;
                this._currentHoveringClientID = clientID;
                const sheetsPopupService = this._injector.get(
                    SheetCanvasPopManagerService
                );

                const skeleton = this._renderManagerService
                    .getRenderById(cell.unitId)
                    ?.with(SheetSkeletonManagerService)
                    .getWorksheetSkeleton(cell.subUnitId)
                    ?.skeleton;
                if (!skeleton) return;

                const cellWithCoord = skeleton.getCellWithCoordByIndex(
                    v.selection.range.startRow,
                    v.selection.range.endColumn
                );

                const disposePopup = sheetsPopupService.attachPopupToCell(
                    v.selection.range.startRow,
                    v.selection.range.endColumn,
                    {
                        componentKey: CollaborationSelectionPopup.componentKey,
                        extraProps: {
                            color: this._ensureUserColor(cell.unitId, clientID),
                            name,
                        },
                        offset: [
                            -(cellWithCoord.endX - cellWithCoord.startX) - 5,
                            -(cellWithCoord.endY - cellWithCoord.startY),
                        ],
                    },
                    cell.unitId,
                    cell.subUnitId
                );
                if (disposePopup) {
                    this._disposableCollection.add(disposePopup);
                }
            }
        });
    }

    async _initAwarenessUI() {
        const unitIds = await this._awarenessService.getInitUnitIds();
        unitIds.forEach((unitId) => {
            this.handleAwarenessInit(unitId);
        });

        this.disposeWithMe(
            this._awarenessService.init$.subscribe((map) => {
                map.keys().forEach((unitId) => {
                    this.handleAwarenessInit(unitId);
                });
            })
        );
    }

    async handleAwarenessInit(unitId: string) {
        if (this._initUnitIds.has(unitId)) return;
        this._initUnitIds.add(unitId);
        const shapeMap = this._markSelectionService.getShapeMap();

        const prefix = 'collab-';
        const updateAwarenessState = async (clientID: number) => {
            const isSelf =
                clientID === (await this._awarenessService.getClientId(unitId));
            if (isSelf) return;

            const shapeMap = this._markSelectionService.getShapeMap();
            shapeMap.forEach((_v, k) => {
                if (k.startsWith(`${prefix}${clientID.toString()}`)) {
                    this._markSelectionService.removeShape(k);
                }
            });
            const states = await this._awarenessService.getState(unitId);
            if (!states) return;
            const awarenessState = states.get(clientID);
            if (!awarenessState) return;
            const { selectionParams } = awarenessState as IAwarenessState;
            selectionParams?.selections.forEach((selection, index) => {
                const id = `${prefix}${clientID.toString()}-${index}`;
                const markSelectionInfo: IMarkSelectionInfo = {
                    selection: {
                        ...selection,
                        style: {
                            ...selection.style,
                            stroke: this._ensureUserColor(unitId, clientID),
                        },
                    },
                    subUnitId: selectionParams.subUnitId,
                    unitId: selectionParams.unitId,
                    zIndex: -1,
                    control: null,
                    exits: [],
                };
                shapeMap.set(id, markSelectionInfo);
                this._markSelectionService.refreshShapes();

                if (this._currentHoveringClientID === clientID) {
                    this._onCellHover({
                        row: selection.range.startRow,
                        col: selection.range.endColumn,
                        unitId: selectionParams.unitId,
                        subUnitId: selectionParams.subUnitId,
                    });
                }
            });
        };

        const states = await this._awarenessService.getState(unitId);
        states?.forEach((_state, clientID) => {
            updateAwarenessState(clientID);
        });

        this.disposeWithMe(
            this._awarenessService.add$.subscribe((added) => {
                if (added.unitId !== unitId) return;
                updateAwarenessState(added.clientId);
            })
        );
        this.disposeWithMe(
            this._awarenessService.remove$.subscribe((removed) => {
                if (removed.unitId !== unitId) return;
                if (this._currentHoveringClientID === removed.clientId) {
                    this._currentHoveringClientID = null;
                    this._disposableCollection.dispose();
                }
                shapeMap.forEach((_v, k) => {
                    if (k.startsWith(`${prefix}${removed.clientId.toString()}-`)) {
                        this._markSelectionService.removeShape(k);
                    }
                });
            })
        );
        this.disposeWithMe(
            this._awarenessService.update$.subscribe((updated) => {
                if (updated.unitId !== unitId) return;
                updateAwarenessState(updated.clientId);
            })
        );

        const setSelectionsOperationCallback = throttle(
            (command: Readonly<ICommandInfo>, _options?: IExecutionOptions) => {
                const params = Tools.deepClone(
                    command.params
                ) as ISetSelectionsOperationParams;
                params.selections.forEach((selection) => {
                    selection.style = null;
                    selection.primary = null;
                });
                this._awarenessService.setLocalStateField(
                    unitId,
                    'selectionParams',
                    params
                );
            },
            500,
            {
                leading: true,
                trailing: true,
            }
        );

        this.disposeWithMe(
            this._commandService.onCommandExecuted((command, options) => {
                if (command.id === SetSelectionsOperation.id) {
                    setSelectionsOperationCallback(command, options);
                }
            })
        );
    }

    private _ensureUserColor(unitId: string, clientID: number) {
        let colorMap = this._userColorMap.get(unitId);
        if (!colorMap) {
            colorMap = new Map();
            this._userColorMap.set(unitId, colorMap);
        }
        if (!colorMap.has(clientID)) {
            colorMap.set(clientID, randomColor());
        }
        return colorMap.get(clientID);
    }
}
