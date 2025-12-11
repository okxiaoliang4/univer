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

import type { ICellData, IObjectArrayPrimitiveType, IObjectMatrixPrimitiveType, Nullable } from '@univerjs/core';
import type {
    IAxisItem,
    IAxisItemType,
    IAxisModel,
    IPivotField,
    IPivotFilterCriteria,
    IPivotModel,
    IPivotSortDirection,
    IPivotTableCrossTabConfig,
} from '../types/type';
import { Disposable, ObjectMatrix } from '@univerjs/core';
import { createAggregator } from '../common/aggregation/functions';
import { defaultPlaceholderMatrix } from '../common/default-pivot-table';
import { AggregationType } from '../types/enum';

const BLANK = '(blank)';
const SEP = '|';
const toKey = (vals: string[]): string => vals.join(SEP);

interface IProcessedRow {
    row: ICellData[];
    rowKey: string;
    rowPath: string[];
    colKey: string;
    colPath: string[];
}

interface IAxisBuildResult {
    model: IAxisModel;
    comboSets: string[][];
}

export type PivotCellType = 'header' | 'rowHeader' | 'columnHeader' | 'data' | 'subtotal' | 'grandTotal';
interface IPivotRenderCellInfo {
    type: PivotCellType;
    level: number;
}

export class PivotEngineV3 extends Disposable {
    private _rowFields: IPivotField[];
    private _columnFields: IPivotField[];
    private _valueFields: IPivotField[];
    private _filterFields: IPivotField[];
    private _sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>;
    private _calculatedData: IPivotModel | null = null;
    private _isDirty = true;

    constructor(config: IPivotTableCrossTabConfig) {
        super();
        this._rowFields = config.rowFields;
        this._columnFields = config.columnFields;
        this._valueFields = config.valueFields;
        this._filterFields = config.filterFields;
        this._sourceData = config.sourceData;
    }

    getPivotModel(): IPivotModel {
        if (this._isDirty || !this._calculatedData) {
            this._calculatedData = this._calculate();
            this._isDirty = false;
        }
        return this._calculatedData;
    }

    getOutputMatrix(): IObjectMatrixPrimitiveType<Nullable<ICellData>> | null {
        const model = this.getPivotModel();
        if (model.isEmpty) return defaultPlaceholderMatrix;
        return this._buildMatrixFromModel(model);
    }

    getRowFields(): IPivotField[] {
        return this._rowFields;
    }

    getColumnFields(): IPivotField[] {
        return this._columnFields;
    }

    getValueFields(): IPivotField[] {
        return this._valueFields;
    }

    getFilterFields(): IPivotField[] {
        return this._filterFields;
    }

    getRowInfo(rowIndex: number) {
        const pivotModel = this.getPivotModel();
        const item = pivotModel.rowAxis.items[rowIndex];
        if (!item) {
            return { headers: [], type: 'data' as IAxisItemType, level: 0, fieldIndex: 0 };
        }
        return {
            headers: this._normalizeHeaders(item.headers, pivotModel.rowAxis.headerDepth),
            type: item.type,
            level: item.level,
            fieldIndex: item.fieldIndex,
        };
    }

    getColumnInfo(columnIndex: number) {
        const pivotModel = this.getPivotModel();
        const item = pivotModel.colAxis.items[columnIndex];
        if (!item) {
            return { headers: [], type: 'data' as IAxisItemType, level: 0, fieldIndex: 0 };
        }
        return {
            headers: this._normalizeHeaders(item.headers, pivotModel.colAxis.headerDepth),
            type: item.type,
            level: item.level,
            fieldIndex: item.fieldIndex,
        };
    }

    getCellInfo(rowIndex: number, columnIndex: number) {
        const pivotModel = this.getPivotModel();
        const rowInfo = this.getRowInfo(rowIndex);
        const columnInfo = this.getColumnInfo(columnIndex);
        const cell = pivotModel.values?.[rowIndex]?.[columnIndex];
        const firstValue = cell ? cell[Number(Object.keys(cell)[0] ?? '0')] ?? null : null;
        return {
            value: firstValue as number | string | null,
            rowInfo,
            columnInfo,
        };
    }

    getHeaderRowsCount(): number {
        const model = this.getPivotModel();
        if (model.isEmpty) return 0;
        const columnHeaderDepth = model.colAxis.headerDepth;
        let rowHeaderDepth = model.rowAxis.headerDepth;
        const hasValueHeaderRow = this._columnFields.length > 0 && this._valueFields.length > 1;

        if (
            rowHeaderDepth === 0 &&
            this._rowFields.length === 0 &&
            this._columnFields.length > 0 &&
            this._valueFields.length === 1
        ) {
            rowHeaderDepth = 1;
        }

        if (this._columnFields.length > 0) {
            return 1 + columnHeaderDepth + (hasValueHeaderRow ? 1 : 0);
        }

        return rowHeaderDepth > 0 ? 1 : 0;
    }

    determineCellType(rowIndex: number, columnIndex: number): IPivotRenderCellInfo {
        const pivotModel = this.getPivotModel();

        // Header depths
        let rowHeaderDepth = pivotModel.rowAxis.headerDepth;
        const columnHeaderDepth = pivotModel.colAxis.headerDepth;

        // Align with _buildStructureFromModel/_buildMatrix special case:
        // when no row fields, but have column fields and single value field, we add a value row header.
        if (
            rowHeaderDepth === 0 &&
            this._rowFields.length === 0 &&
            this._columnFields.length > 0 &&
            this._valueFields.length === 1
        ) {
            rowHeaderDepth = 1;
        }

        const hasValueHeaderRow = this._columnFields.length > 0 && this._valueFields.length > 1;
        const headerRowsCount = this._columnFields.length > 0
            ? 1 + columnHeaderDepth + (hasValueHeaderRow ? 1 : 0) // top field-name row + column header rows + optional value header row
            : 1; // when no column fields we still render a single header row

        // Header area
        if (rowIndex < headerRowsCount) {
            return { type: 'header', level: Math.min(rowIndex, columnHeaderDepth) };
        }

        // Data area
        const dataRowIndex = rowIndex - headerRowsCount;
        if (
            dataRowIndex < 0 ||
            dataRowIndex >= pivotModel.rowAxis.items.length
        ) {
            return { type: 'data', level: 0 };
        }

        const rowInfo = this.getRowInfo(dataRowIndex);

        // Row header columns (left side) before hitting data columns
        if (columnIndex < rowHeaderDepth) {
            return { type: 'rowHeader', level: Math.max(0, rowInfo.level) };
        }

        const logicalColumnIndex = columnIndex - rowHeaderDepth;
        if (
            logicalColumnIndex < 0 ||
            logicalColumnIndex >= pivotModel.colAxis.items.length
        ) {
            return { type: 'data', level: 0 };
        }

        const colInfo = this.getColumnInfo(logicalColumnIndex);

        if (rowInfo.type === 'grand' || colInfo.type === 'grand') {
            return { type: 'grandTotal', level: 0 };
        }

        if (rowInfo.type === 'subtotal' || colInfo.type === 'subtotal') {
            const level = this._getSubtotalLevelFromModel(
                pivotModel.rowAxis.items[dataRowIndex],
                pivotModel.colAxis.items[logicalColumnIndex]
            );
            return { type: 'subtotal', level };
        }

        return { type: columnIndex < rowHeaderDepth ? 'rowHeader' : 'data', level: Math.max(0, rowInfo.level) };
    }

    setRowFields(rowFields: IPivotField[]): void {
        this._rowFields = rowFields;
        this._isDirty = true;
    }

    setColumnFields(columnFields: IPivotField[]): void {
        this._columnFields = columnFields;
        this._isDirty = true;
    }

    setValueFields(valueFields: IPivotField[]): void {
        this._valueFields = valueFields;
        this._isDirty = true;
    }

    setFilterFields(filterFields: IPivotField[]): void {
        this._filterFields = filterFields;
        this._isDirty = true;
    }

    setSourceData(sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>): void {
        this._sourceData = sourceData;
        this._isDirty = true;
    }

    setCalculatedData(data: IPivotModel, isDirty = true): void {
        this._calculatedData = data;
        this._isDirty = isDirty;
    }

    private _calculate(): IPivotModel {
        if (this._rowFields.length === 0 && this._columnFields.length === 0 && this._valueFields.length === 0) {
            return this._emptyResult();
        }

        const filteredRows = this._filterRows();
        const rowIdx = this._rowFields.map((f) => f.sourceColumnIndex);
        const colIdx = this._columnFields.map((f) => f.sourceColumnIndex);
        const valIdx = this._valueFields.map((f) => f.sourceColumnIndex);
        const processed = this._processRows(filteredRows, rowIdx, colIdx);

        const pivotModel = this._buildPivotModel(processed, rowIdx, colIdx, valIdx);
        return pivotModel;
    }

    private _buildPivotModel(
        processedRows: IProcessedRow[],
        rowIndices: number[],
        colIndices: number[],
        valueIndices: number[]
    ): IPivotModel {
        const rowCombos = this._uniquePaths(processedRows, 'row', this._rowFields.length);
        const colCombos = this._uniquePaths(processedRows, 'column', this._columnFields.length);
        const rowKeysAll = Array.from(new Set(processedRows.map((r) => r.rowKey)));
        const colKeysAll = Array.from(new Set(processedRows.map((r) => r.colKey)));

        const rowAxisResult = this._buildAxisModel('row', this._rowFields, rowCombos, rowKeysAll, colKeysAll);
        const colAxisResult = this._buildAxisModel('column', this._columnFields, colCombos, rowKeysAll, colKeysAll);

        const values = this._buildValues(
            colAxisResult.model.items,
            rowAxisResult.comboSets,
            colAxisResult.comboSets,
            processedRows,
            valueIndices
        );
        const hasValues = Object.values(values || {}).some((row) =>
            Object.values(row || {}).some((cell) =>
                Object.values(cell || {}).some((v) => v !== null && v !== undefined && v !== '')
            )
        );

        const isEmpty = !(hasValues || rowAxisResult.model.items.length > 0 || colAxisResult.model.items.length > 0 || this._valueFields.length > 0);

        return {
            isEmpty,
            valueFields: this._valueFields.map((f) => ({
                id: f.id,
                name: f.name,
                agg: f.aggregation || AggregationType.SUM,
            })),
            rowAxis: rowAxisResult.model,
            colAxis: colAxisResult.model,
            values,
            dimensions: {
                rowCount: rowAxisResult.model.items.length,
                colCount: colAxisResult.model.items.length,
                valueFieldCount: this._valueFields.length,
            },
        };
    }

    private _buildAxisModel(
        kind: 'row' | 'column',
        fields: IPivotField[],
        combos: string[][],
        rowKeysAll: string[],
        colKeysAll: string[]
    ): IAxisBuildResult {
        const baseDepth = fields.length;
        const headerDepth = baseDepth > 0 ? baseDepth : (kind === 'column' ? 1 : 0);
        const hasFields = baseDepth > 0;

        const hasGrand = fields[0]?.showSubTotals === true;

        if (!hasFields) {
            const items: IAxisModel['items'] = [];
            const comboSets: string[][] = [];

            if (kind === 'column') {
                this._valueFields.forEach((_, vIdx) => {
                    items.push({
                        headers: [this._label(this._valueFields[vIdx])],
                        display: [],
                        type: 'data',
                        level: 0,
                        fieldIndex: 0,
                        valueFieldIndex: vIdx,
                        groupKey: `${kind}:data:root:v${vIdx}`,
                    });
                    comboSets.push(colKeysAll);
                });
            } else {
                items.push({
                    headers: [],
                    display: [],
                    type: 'data',
                    level: 0,
                    fieldIndex: 0,
                    groupKey: `${kind}:data:root`,
                });
                comboSets.push(rowKeysAll);
            }

            this._fillDisplay(items, headerDepth);
            return {
                model: {
                    items,
                    headerDepth,
                    levelMap: { 0: items.map((_, idx) => idx) },
                    subtotalMap: {},
                },
                comboSets,
            };
        }

        interface Node {
            key: string;
            level: number;
            path: string[];
            children: Map<string, Node>;
            rowKeys: Set<string>;
            colKeys: Set<string>;
        }

        const root: Node = {
            key: 'root',
            level: -1,
            path: [],
            children: new Map(),
            rowKeys: new Set<string>(),
            colKeys: new Set<string>(),
        };

        combos.forEach((path) => {
            const comboKey = toKey(path);
            let node = root;
            node.rowKeys.add(comboKey);
            node.colKeys.add(comboKey);
            for (let level = 0; level < headerDepth; level++) {
                const val = path[level] ?? BLANK;
                if (!node.children.has(val)) {
                    node.children.set(val, {
                        key: val,
                        level,
                        path: [...node.path, val],
                        children: new Map(),
                        rowKeys: new Set<string>(),
                        colKeys: new Set<string>(),
                    });
                }
                node = node.children.get(val)!;
                node.rowKeys.add(comboKey);
                node.colKeys.add(comboKey);
            }
        });

        const items: IAxisModel['items'] = [];
        const comboSets: string[][] = [];
        const levelMap: Record<number, number[]> = {};
        const subtotalMap: Record<number, number[]> = {};
        const seenGroupKeys = new Set<string>();

        const sortKeys = (children: Node[], level: number): Node[] => {
            const sortRule = fields[level]?.sortRule;
            const dir = sortRule?.direction || 'asc';
            return children.sort((a, b) => this._cmp(a.key, b.key, dir));
        };

        const addItem = (item: IAxisModel['items'][number], combosForItem: string[]) => {
            if (seenGroupKeys.has(item.groupKey)) return;
            seenGroupKeys.add(item.groupKey);
            const idx = items.length;
            items.push(item);
            comboSets.push(combosForItem);
            if (!levelMap[item.level]) levelMap[item.level] = [];
            levelMap[item.level].push(idx);
            if (item.type !== 'data') {
                if (!subtotalMap[item.level]) subtotalMap[item.level] = [];
                subtotalMap[item.level].push(idx);
            }
        };

        const traverse = (node: Node) => {
            const level = node.level;
            const children = sortKeys(Array.from(node.children.values()), level + 1);

            if (children.length === 0 && level === headerDepth - 1) {
                const headers = node.path.map((v) => (v === BLANK ? '' : v));
                const combosForItem = Array.from(kind === 'row' ? node.rowKeys : node.colKeys);
                if (kind === 'column') {
                    if (this._valueFields.length === 0) {
                        addItem(
                            {
                                headers,
                                display: [],
                                type: 'data',
                                level,
                                fieldIndex: level,
                                groupKey: `${kind}:data:${toKey(headers)}`,
                            },
                            combosForItem
                        );
                    } else {
                        this._valueFields.forEach((_, vIdx) => {
                            addItem(
                                {
                                    headers,
                                    display: [],
                                    type: 'data',
                                    level,
                                    fieldIndex: level,
                                    valueFieldIndex: vIdx,
                                    groupKey: `${kind}:data:${toKey(headers)}:v${vIdx}`,
                                },
                                combosForItem
                            );
                        });
                    }
                } else {
                    addItem(
                        {
                            headers,
                            display: [],
                            type: 'data',
                            level,
                            fieldIndex: level,
                            groupKey: `${kind}:data:${toKey(headers)}`,
                        },
                        combosForItem
                    );
                }
            } else {
                for (const child of children) {
                    traverse(child);
                }
            }

            // subtotal for parent level if child field has showSubTotals
            const childField = fields[level + 1];
            const needSubtotal = childField?.showSubTotals === true && level >= 0;
            if (needSubtotal) {
                const headers = new Array(headerDepth).fill('');
                for (let i = 0; i <= level; i++) headers[i] = node.path[i] === BLANK ? '' : node.path[i];
                const labelBase = headers[level] || '';
                headers[level] = labelBase ? `${labelBase} 总计` : '总计';
                const combosForSubtotal = Array.from(kind === 'row' ? node.rowKeys : node.colKeys);
                if (kind === 'row') {
                    const groupKey = `${kind}:subtotal:${toKey(node.path.slice(0, level + 1))}`;
                    addItem(
                        {
                            headers,
                            display: [],
                            type: 'subtotal',
                            level,
                            fieldIndex: level,
                            groupKey,
                        },
                        combosForSubtotal
                    );
                } else {
                    this._valueFields.forEach((_, vIdx) => {
                        const groupKey = `${kind}:subtotal:${toKey(headers)}:${vIdx}`;
                        addItem(
                            {
                                headers,
                                display: [],
                                type: 'subtotal',
                                level,
                                fieldIndex: level,
                                valueFieldIndex: vIdx,
                                groupKey,
                            },
                            combosForSubtotal
                        );
                    });
                }
            }
        };

        const topChildren = sortKeys(Array.from(root.children.values()), 0);
        for (const child of topChildren) {
            traverse(child);
        }

        if (hasGrand) {
            const headers = new Array(headerDepth).fill('');
            headers[0] = '总计';
            const combosForGrand = kind === 'row' ? rowKeysAll : colKeysAll;
            if (kind === 'column' && this._valueFields.length === 0) {
                addItem(
                    {
                        headers,
                        display: [],
                        type: 'grand',
                        level: 0,
                        fieldIndex: 0,
                        groupKey: `${kind}:grand`,
                    },
                    combosForGrand
                );
            } else if (kind === 'column') {
                this._valueFields.forEach((_, vIdx) => {
                    addItem(
                        {
                            headers,
                            display: [],
                            type: 'grand',
                            level: 0,
                            fieldIndex: 0,
                            valueFieldIndex: vIdx,
                            groupKey: `${kind}:grand:${vIdx}`,
                        },
                        combosForGrand
                    );
                });
            } else {
                addItem(
                    {
                        headers,
                        display: [],
                        type: 'grand',
                        level: 0,
                        fieldIndex: 0,
                        groupKey: `${kind}:grand`,
                    },
                    combosForGrand
                );
            }
        }

        this._fillDisplay(items, headerDepth);

        return {
            model: {
                items,
                headerDepth,
                levelMap,
                subtotalMap,
            },
            comboSets,
        };
    }

    private _buildValues(
        colItems: IAxisModel['items'],
        rowComboSets: string[][],
        colComboSets: string[][],
        processedRows: IProcessedRow[],
        valueIndices: number[]
    ): IObjectMatrixPrimitiveType<IObjectArrayPrimitiveType<number | string | null>> {
        const map = new Map<string, Map<string, ICellData[][]>>();
        for (const pr of processedRows) {
            if (!map.has(pr.rowKey)) map.set(pr.rowKey, new Map());
            const colMap = map.get(pr.rowKey)!;
            if (!colMap.has(pr.colKey)) colMap.set(pr.colKey, []);
            colMap.get(pr.colKey)!.push(pr.row);
        }

        const values: IObjectMatrixPrimitiveType<IObjectArrayPrimitiveType<number | string | null>> = {};

        rowComboSets.forEach((rowCombos, rIdx) => {
            const rowVal: IObjectArrayPrimitiveType<IObjectArrayPrimitiveType<number | string | null>> = {};
            colComboSets.forEach((colCombos, cIdx) => {
                const cellVals: IObjectArrayPrimitiveType<number | string | null> = {};
                const targetValueIdx = colItems[cIdx]?.valueFieldIndex;
                for (let vIdx = 0; vIdx < valueIndices.length; vIdx++) {
                    if (targetValueIdx !== undefined && targetValueIdx !== vIdx) continue;
                    const aggType = this._valueFields[vIdx]?.aggregation || AggregationType.SUM;
                    const aggregator = createAggregator(aggType);
                    aggregator.init();
                    for (const rk of rowCombos) {
                        const colMap = map.get(rk);
                        if (!colMap) continue;
                        for (const ck of colCombos) {
                            const list = colMap.get(ck);
                            if (!list) continue;
                            for (const row of list) {
                                const cell = row[valueIndices[vIdx]];
                                if (cell) aggregator.addValue(cell);
                            }
                        }
                    }
                    cellVals[vIdx] = (aggregator.getResult() ?? null) as number | string | null;
                }
                rowVal[cIdx] = cellVals;
            });
            values[rIdx] = rowVal;
        });

        return values;
    }

    private _processRows(rows: ICellData[][], rowIdx: number[], colIdx: number[]): IProcessedRow[] {
        return rows.map((row) => {
            const rowPath = rowIdx.map((i) => row[i]?.v?.toString() ?? BLANK);
            const colPath = colIdx.map((i) => row[i]?.v?.toString() ?? BLANK);
            return {
                row,
                rowKey: toKey(rowPath),
                rowPath,
                colKey: toKey(colPath),
                colPath,
            };
        });
    }

    private _uniquePaths(processed: IProcessedRow[], kind: 'row' | 'column', depth: number): string[][] {
        const set = new Set<string>();
        const res: string[][] = [];
        if (depth === 0) return [[]];
        processed.forEach((p) => {
            const path = kind === 'row' ? p.rowPath.slice(0, depth) : p.colPath.slice(0, depth);
            const key = toKey(path);
            if (!set.has(key)) {
                set.add(key);
                res.push(path);
            }
        });
        return res;
    }

    private _fillDisplay(items: IAxisModel['items'], depth: number): void {
        const last: string[] = new Array(depth).fill('');
        items.forEach((item) => {
            const disp: string[] = [];
            for (let i = 0; i < depth; i++) {
                const v = item.headers[i] ?? '';
                disp[i] = v === last[i] ? '' : v;
                if (v !== '') last[i] = v;
            }
            item.display = disp;
        });
    }

    private _normalizeHeaders(headers: string[], depth: number): string[] {
        const res = new Array(depth).fill('');
        headers.forEach((h, idx) => {
            if (idx < depth) res[idx] = h ?? '';
        });
        return res;
    }

    private _buildMatrixFromModel(pivotModel: IPivotModel): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
        let rowDepth = pivotModel.rowAxis.headerDepth;
        const colDepth = pivotModel.colAxis.headerDepth;
        const needValueRowHeader =
            rowDepth === 0 && this._rowFields.length === 0 && this._columnFields.length > 0 && this._valueFields.length === 1;

        if (needValueRowHeader) {
            rowDepth = 1;
        }

        let rowHeaders = pivotModel.rowAxis.items.map((i) => this._normalizeHeaders(i.headers, rowDepth));
        if (needValueRowHeader) {
            rowHeaders = rowHeaders.map(() => [this._label(this._valueFields[0])]);
        }
        const columnHeaders = pivotModel.colAxis.items.map((i) => this._normalizeHeaders(i.headers, colDepth));

        const valueFieldHeaders = this._valueFields.length > 1 ? this._valueFields.map((f) => this._label(f)) : undefined;
        const values = pivotModel.values;

        const matrix = new ObjectMatrix<Nullable<ICellData>>();
        const rowHeaderDepth = rowHeaders[0]?.length ?? 0;
        const columnHeaderDepth = Math.max(...columnHeaders.map((h) => h.length), 0);

        let r = 0;

        // No column fields: render a simple header row with row field names + value labels
        if (this._columnFields.length === 0) {
            const row: Nullable<ICellData>[] = [];
            for (let i = 0; i < rowHeaderDepth; i++) row.push({ v: this._rowFields[i]?.name || '' });
            const labels = this._valueFields.length > 0
                ? (valueFieldHeaders || this._valueFields.map((f) => this._label(f)))
                : [];
            labels.forEach((v) => row.push({ v }));
            row.forEach((cell, c) => {
                if (cell?.v) matrix.setValue(r, c, cell);
            });
            r++;
        }

        if (columnHeaderDepth > 0 && this._columnFields.length > 0) {
            const row: Nullable<ICellData>[] = [];
            for (let i = 0; i < rowHeaderDepth; i++) row.push({ v: '' });
            this._columnFields.forEach((f) => row.push({ v: f.name }));
            if (this._valueFields.length > 1) row.push({ v: '值' });
            row.forEach((cell, c) => {
                if (cell?.v) matrix.setValue(r, c, cell);
            });
            r++;
        }

        for (let level = 0; level < columnHeaderDepth && this._columnFields.length > 0; level++) {
            const row: Nullable<ICellData>[] = [];
            const isLast = level === columnHeaderDepth - 1;
            const showRowNames = isLast && this._valueFields.length <= 1;
            for (let i = 0; i < rowHeaderDepth; i++) row.push({ v: showRowNames ? this._rowFields[i]?.name || '' : '' });
            let prevHeaders: string[] | null = null;
            for (let c = 0; c < columnHeaders.length; c++) {
                const hdr = columnHeaders[c];
                const v = hdr[level] ?? '';
                let show = true;
                if (prevHeaders) {
                    let sameUpper = true;
                    for (let k = 0; k < level; k++) {
                        if ((hdr[k] ?? '') !== (prevHeaders[k] ?? '')) {
                            sameUpper = false;
                            break;
                        }
                    }
                    if (sameUpper && v === (prevHeaders[level] ?? '')) {
                        show = false;
                    }
                }
                row.push({ v: show ? v : '' });
                prevHeaders = hdr;
            }
            row.forEach((cell, c) => {
                if (cell?.v) matrix.setValue(r, c, cell);
            });
            r++;
        }

        if (this._valueFields.length > 1 && this._columnFields.length > 0) {
            const row: Nullable<ICellData>[] = [];
            for (let i = 0; i < rowHeaderDepth; i++) row.push({ v: this._rowFields[i]?.name || '' });
            const labels = valueFieldHeaders || this._valueFields.map((f) => this._label(f));
            for (let c = 0; c < columnHeaders.length; c++) {
                row.push({ v: labels[c % labels.length] });
            }
            row.forEach((cell, c) => {
                if (cell?.v) matrix.setValue(r, c, cell);
            });
            r++;
        }

        const lastHdrs: string[] = new Array(rowHeaderDepth).fill('');
        rowHeaders.forEach((hdr, rowIdx) => {
            const row: Nullable<ICellData>[] = [];
            for (let i = 0; i < rowHeaderDepth; i++) {
                const v = hdr[i] ?? '';
                const show = rowIdx === 0 || v !== lastHdrs[i];
                row.push({ v: show ? v : '' });
                if (show) lastHdrs[i] = v;
            }
            const rowVals = values[rowIdx] || {};
            for (let c = 0; c < columnHeaders.length; c++) {
                const cell = rowVals[c];
                const first = cell ? cell[Number(Object.keys(cell)[0] ?? '0')] : '';
                row.push({ v: first ?? '' });
            }
            row.forEach((cell, c) => {
                if (cell?.v !== '' && cell?.v !== null && cell?.v !== undefined) matrix.setValue(r, c, cell);
            });
            r++;
        });

        return matrix.getMatrix();
    }

    private _getSubtotalLevelFromModel(rowItem: IAxisItem, colItem: IAxisItem): number {
        if (rowItem.type === 'subtotal') return rowItem.level;
        if (colItem.type === 'subtotal') return colItem.level;
        return 0;
    }

    private _filterRows(): ICellData[][] {
        const res: ICellData[][] = [];
        const rowKeys = Object.keys(this._sourceData).map(Number).sort((a, b) => a - b);
        for (const rk of rowKeys) {
            if (rk === 0) continue;
            const rowObj = this._sourceData[rk];
            if (!rowObj) continue;
            const cols = Object.keys(rowObj).map(Number);
            const maxCol = Math.max(...cols, 0);
            const row: ICellData[] = [];
            for (let c = 0; c <= maxCol; c++) row[c] = rowObj[c] ?? { v: '' };
            if (this._filterFields.every((f) => this._passes(row, f.filter, f.sourceColumnIndex))) res.push(row);
        }
        return res;
    }

    private _passes(row: ICellData[], filter: IPivotFilterCriteria | undefined, idx?: number): boolean {
        if (!filter) return true;
        const v = row[idx ?? 0]?.v;
        if (filter.type === 'value') return filter.values?.includes(v as string) ?? true;
        const cv = filter.conditionValue;
        switch (filter.operator) {
            case 'equals': return v === cv;
            case 'notEquals': return v !== cv;
            case 'greaterThan': return Number(v) > Number(cv);
            case 'lessThan': return Number(v) < Number(cv);
            case 'contains': return String(v).includes(String(cv));
            default: return true;
        }
    }

    private _cmp(a: string | number, b: string | number, dir: IPivotSortDirection): number {
        const locale = 'zh';
        return dir === 'asc'
            ? String(a).localeCompare(String(b), locale, { numeric: true })
            : String(b).localeCompare(String(a), locale, { numeric: true });
    }

    private _label(field: IPivotField): string {
        const agg = field.aggregation || AggregationType.SUM;
        return this._getAggregationLabel(agg, field.name);
    }

    private _getAggregationLabel(aggregation: AggregationType, name: string): string {
        const map: Record<AggregationType, string> = {
            [AggregationType.SUM]: `用于“${name}”的 SUM`,
            [AggregationType.COUNT]: `用于“${name}”的 COUNT`,
            [AggregationType.AVERAGE]: `用于“${name}”的 AVERAGE`,
            [AggregationType.MAX]: `用于“${name}”的 MAX`,
            [AggregationType.MIN]: `用于“${name}”的 MIN`,
        };
        return map[aggregation] ?? name;
    }

    private _emptyResult(): IPivotModel {
        return {
            isEmpty: true,
            valueFields: [],
            rowAxis: { items: [], headerDepth: 0, levelMap: {}, subtotalMap: {} },
            colAxis: { items: [], headerDepth: 0, levelMap: {}, subtotalMap: {} },
            values: {},
            dimensions: { rowCount: 0, colCount: 0, valueFieldCount: 0 },
        };
    }
}
