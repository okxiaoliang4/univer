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

import type { IColumnData, IObjectArrayPrimitiveType, IObjectMatrixPrimitiveType, IRange, IRowData, Workbook } from '@univerjs/core';
import type { Paths } from 'type-fest';
import {
    cloneDeep,
    getColumnIdFromWorkbook,
    getColumnIndexFromWorkbook,
    getRowIdFromWorkbook,
    getRowIndexFromWorkbook,
    setProp,
} from '@pagepeek/excel-shared';
import { SHEET_PAGEPEEK_CHART_PLUGIN } from '@pagepeek/excel-univer-charts-plugin';
import {

    ObjectMatrix,

} from '@univerjs/core';
import { SHEET_DRAWING_PLUGIN } from '@univerjs/sheets-drawing';
import { get } from 'lodash-es';
import * as Y from 'yjs';

export function getCursorSVG(color: string) {
    return `<svg version="1.1" xmlns="http://www.w3.org/2000/svg"
  xmlns:xlink="http://www.w3.org/1999/xlink" x="0px" y="0px" viewBox="0 0 28 28" xml:space="preserve">
  <polygon fill="#FFFFFF" points="8.2,20.9 8.2,4.9 19.8,16.5 13,16.5 12.6,16.6 "/>
  <polygon fill="#FFFFFF" points="17.3,21.6 13.7,23.1 9,12 12.7,10.5 "/>
  <rect fill="${color}" x="12.5" y="13.6" transform="matrix(0.9221 -0.3871 0.3871 0.9221 -5.7605 6.5909)" width="2" height="8"/>
  <polygon fill="${color}" points="9.2,7.3 9.2,18.5 12.2,15.6 12.6,15.5 17.4,15.5 "/>
</svg>`;
}

export function getIdIndexMap(
    data: IObjectArrayPrimitiveType<Partial<IColumnData | IRowData>>
) {
    const map = new Map<string | number | undefined | null, number>();
    Object.entries(data).forEach(([k, v]) => {
        const id = v.custom?.id;
        if (id) map.set(id, Number(k));
    });
    return map;
}

export function getIdsOfDeletedItems(
    items: Set<Y.Item>,
    transaction: Y.Transaction
) {
    const maps = new Set<unknown>();
    for (const t of items) {
        for (const child of t.content.getContent()) maps.add(child);
    }

    const foundIds = new Set<string>();
    const error = new Error('found enough');
    try {
        let count = 0;
        Y.iterateDeletedStructs(transaction, transaction.deleteSet, (item) => {
            if (count === items.size) throw error;
            if (item.deleted && item instanceof Y.Item && item.parentSub === 'id') {
                if (maps.has(get(item, 'parent.parent'))) {
                    count++;
                    for (const id of item.content.getContent()) foundIds.add(id);
                }
            }
        });
    } catch (e) {
        if (e !== error) throw e;
    }
    return foundIds;
}

export function getChartIdsOfDeletedItems(
    items: Set<Y.Item>,
    transaction: Y.Transaction
) {
    const maps = new Set<unknown>();
    for (const t of items) {
        for (const child of t.content.getContent()) maps.add(child);
    }

    const foundIds = new Set<string>();
    const error = new Error('found enough');
    try {
        let count = 0;
        Y.iterateDeletedStructs(transaction, transaction.deleteSet, (item) => {
            if (count === items.size) throw error;
            if (item.deleted && item instanceof Y.Item && item.parentSub === 'id') {
                if (maps.has(get(item, 'parent'))) {
                    count++;
                    for (const id of item.content.getContent()) foundIds.add(id);
                }
            }
        });
    } catch (e) {
        if (e !== error) throw e;
    }
    return foundIds;
}

type ITransformPathConfig<T extends Record<string | number, any>> = Array<
  | {
      paths: Paths<T>[];
      type: 'range';
  }
  | {
      paths: Paths<T>[];
      type: 'ranges';
  }
  | {
      paths: Paths<T>[];
      type: 'rangesMap';
  }
  | {
      paths: Paths<T>[];
      type: 'rowPaths';
  }
  | {
      paths: Paths<T>[];
      type: 'columnPaths';
  }
  | {
      paths: Paths<T>[];
      type: 'matrix';
  }
>;

export function transformPositionFromPathByConfig<
    T extends Record<string | number, any> & {
        ranges?: IRange[];
    }
>(options: {
    sheetId: string;
    source: T;
    config: ITransformPathConfig<T>;
    ydoc: Y.Doc;
    workbook: Workbook;
}) {
    const { source, sheetId, config, workbook } = options;

  // TODO: 纯函数,为性能的话后面可以改掉clone,直接操作源对象
    const result: T = cloneDeep(source);

    const processRange = (range: Partial<IRange>) => {
        const { startRow, endRow, startColumn, endColumn } = range;
    // TODO: 针对了column，加多个配置
        range.startRow =
            typeof startRow === 'number'
                ? getRowIdFromWorkbook(startRow, sheetId, workbook)
                : undefined;
        range.endRow =
            typeof endRow === 'number'
                ? getRowIdFromWorkbook(endRow, sheetId, workbook)
                : undefined;
        range.startColumn =
            typeof startColumn === 'number'
                ? getColumnIdFromWorkbook(startColumn, sheetId, workbook)
                : undefined;
        range.endColumn =
            typeof endColumn === 'number'
                ? getColumnIdFromWorkbook(endColumn, sheetId, workbook)
                : undefined;
    };

    const processMatrix = (object: IObjectMatrixPrimitiveType<T>) => {
        const matrix = new ObjectMatrix(object);
        matrix.forValue((row, col, value) => {
            matrix.realDeleteValue(row, col);
            matrix.setValue(
                getRowIdFromWorkbook(row, sheetId, workbook),
                getColumnIdFromWorkbook(col, sheetId, workbook),
                value
            );
        });
    };

    config.forEach((item) => {
        const { paths, type } = item;
        if (type === 'rowPaths') {
            paths.forEach((path) => {
                const v = get(result, path);
                if (!v) return;
        // @ts-expect-error types
                setProp(result, path, getRowIdFromWorkbook(v, sheetId, workbook));
            });
        } else if (type === 'columnPaths') {
            paths.forEach((path) => {
                const v = get(result, path);
                if (!v) return;
        // @ts-expect-error types
                setProp(result, path, getColumnIdFromWorkbook(v, sheetId, workbook));
            });
        } else if (type === 'ranges') {
            paths.forEach((path) => {
                const ranges = get(result, path) as IRange[];
                ranges.forEach((range) => {
                    processRange(range);
                });
            });
        } else if (type === 'range') {
            paths.forEach((path) => {
                processRange(get(result, path));
            });
        } else if (type === 'rangesMap') {
            paths.forEach((path) => {
                const mapRanges = get(result, path) as Record<string, IRange[]>;
                Object.values(mapRanges).forEach((ranges) => {
                    ranges.forEach((range) => {
                        processRange(range);
                    });
                });
            });
        } else if (type === 'matrix') {
            paths.forEach((path) => {
                processMatrix(get(result, path));
            });
        }
    });
    return result;
}

export function reoveryPositionFromPathByConfig<
    T extends Record<string | number, any> & {
        ranges?: IRange[];
    }
>(options: {
    sheetId: string;
    source: T;
    config: ITransformPathConfig<T>;
    ydoc: Y.Doc;
    workbook: Workbook;
}) {
    const { source, sheetId, config, workbook } = options;

  // TODO: 纯函数,为性能的话后面可以改掉clone,直接操作源对象
    const result: T = cloneDeep(source);

    const processRange = (range: Partial<IRange>) => {
        const { startRow, endRow, startColumn, endColumn } = range;
        range.startRow =
            typeof startRow === 'string'
                ? getRowIndexFromWorkbook(startRow, sheetId, workbook)
                : startRow;
        range.endRow =
            typeof endRow === 'string'
                ? getRowIndexFromWorkbook(endRow, sheetId, workbook)
                : endRow;
        range.startColumn =
            typeof startColumn === 'string'
                ? getColumnIndexFromWorkbook(startColumn, sheetId, workbook)
                : startColumn;
        range.endColumn =
            typeof endColumn === 'string'
                ? getColumnIndexFromWorkbook(endColumn, sheetId, workbook)
                : endColumn;
    };

    const processMatrix = (object: IObjectMatrixPrimitiveType<T>) => {
        const matrix = new ObjectMatrix(object);
        matrix.forValue((row, col, value) => {
            matrix.realDeleteValue(row, col);
            matrix.setValue(
                getRowIndexFromWorkbook(row, sheetId, workbook),
                getColumnIndexFromWorkbook(col, sheetId, workbook),
                value
            );
        });
    };

    config.forEach((item) => {
        const { paths, type } = item;
        if (type === 'rowPaths') {
            paths.forEach((path) => {
                const v = get(result, path);
                if (!v) return;
        // @ts-expect-error types
                setProp(result, path, getRowIndexFromWorkbook(v, sheetId, workbook));
            });
        } else if (type === 'columnPaths') {
            paths.forEach((path) => {
                const v = get(result, path);
                if (!v) return;
                setProp(
                    result,
          // @ts-expect-error types
                    path,
                    getColumnIndexFromWorkbook(v, sheetId, workbook)
                );
            });
        } else if (type === 'ranges') {
            paths.forEach((path) => {
                const ranges = get(result, path) as IRange[];
                ranges.forEach((range) => {
                    processRange(range);
                });
            });
        } else if (type === 'range') {
            paths.forEach((path) => {
                processRange(get(result, path));
            });
        } else if (type === 'rangesMap') {
            paths.forEach((path) => {
                const mapRanges = get(result, path) as Record<string, IRange[]>;
                Object.values(mapRanges).forEach((ranges) => {
                    ranges.forEach((range) => {
                        processRange(range);
                    });
                });
            });
        } else if (type === 'matrix') {
            paths.forEach((path) => {
                processMatrix(get(result, path));
            });
        }
    });
    return result;
}

export function rebuildValue(paths: string[], value: unknown) {
    if (!paths.length) return value;
    const v: { [k: string]: any } = {};
    let parent = v;
    for (let i = 0; i < paths.length; i++) {
        const key = paths[i];
        if (i === paths.length - 1) {
            parent[key] = value;
        } else {
            parent[key] = {};
            parent = parent[key];
        }
    }
    return v;
}

export function getOrCreateChartResourceOfChartId(
    yResources: Y.Map<unknown>,
    unitId: string,
    subUnitId: string,
    chartId: string
) {
    if (!yResources.has(SHEET_PAGEPEEK_CHART_PLUGIN)) {
        yResources.set(SHEET_PAGEPEEK_CHART_PLUGIN, new Y.Map<unknown>());
    }
    const chartResource = yResources.get(
        SHEET_PAGEPEEK_CHART_PLUGIN
    ) as Y.Map<unknown>;
    if (!chartResource.has(unitId)) {
        chartResource.set(unitId, new Y.Map<unknown>());
    }
    const unitChartResource = chartResource.get(unitId) as Y.Map<unknown>;
    if (!unitChartResource.has(subUnitId)) {
        unitChartResource.set(subUnitId, new Y.Array<Y.Map<unknown>>());
    }
    const subUnitChartResource = unitChartResource.get(subUnitId) as Y.Array<
        Y.Map<unknown>
    >;
    const subUnitChartResourceArray = subUnitChartResource.toArray();

    let targetChartResource = subUnitChartResourceArray.find(
        (t) => t.get('id') === chartId
    );

    if (!targetChartResource) {
        targetChartResource = new Y.Map<unknown>();
        subUnitChartResource.push([targetChartResource]);
    }
    return targetChartResource as Y.Map<unknown>;
}

export function getOrCreateDrawingResourceOfDrawingId(
    yResources: Y.Map<unknown>,
    subUnitId: string,
    drawingId: string
) {
    if (!yResources.has(SHEET_DRAWING_PLUGIN)) {
        yResources.set(SHEET_DRAWING_PLUGIN, new Y.Map<unknown>());
    }
    const drawingResource = yResources.get(SHEET_DRAWING_PLUGIN) as Y.Map<unknown>;
    if (!drawingResource.has(subUnitId)) {
        const map = new Y.Map<unknown>();
        const data = new Y.Map<unknown>();
        drawingResource.set(subUnitId, map);
        map.set('data', data);
        map.set('order', new Y.Array());
        data.set(drawingId, new Y.Map<unknown>());
    }
    const subUnitDrawingResource = drawingResource.get(
        subUnitId
    ) as Y.Map<unknown>;

    if (!subUnitDrawingResource.has('data')) {
        const data = new Y.Map<unknown>();
        subUnitDrawingResource.set('data', data);
        data.set(drawingId, new Y.Map<unknown>());
    }

    const data = subUnitDrawingResource.get('data') as Y.Map<unknown>;

    if (!data.has(drawingId)) {
        data.set(drawingId, new Y.Map<unknown>());
    }

    return subUnitDrawingResource;
}

export function getOrCreateDrawingDataResourceOfDrawingId(
    yResources: Y.Map<unknown>,
    subUnitId: string,
    drawingId: string
) {
    const drawingResource = getOrCreateDrawingResourceOfDrawingId(
        yResources,
        subUnitId,
        drawingId
    );
    return (drawingResource.get('data') as Y.Map<unknown>).get(
        drawingId
    ) as Y.Map<unknown>;
}

export function getOrCreateDrawingOrderResourceOfDrawingId(
    yResources: Y.Map<unknown>,
    subUnitId: string,
    drawingId: string
) {
    const drawingResource = getOrCreateDrawingResourceOfDrawingId(
        yResources,
        subUnitId,
        drawingId
    );
    return drawingResource.get('order') as Y.Array<string>;
}

export function deserializeYAbstractType<T>(o: unknown) {
    if (o instanceof Y.AbstractType) {
        return o.toJSON();
    }
    return o;
}
