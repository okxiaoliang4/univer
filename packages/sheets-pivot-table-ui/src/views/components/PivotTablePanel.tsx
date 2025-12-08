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

import type { IDocumentData, Workbook } from '@univerjs/core';
import type { IRangeSelectorInstance } from '@univerjs/sheets-formula-ui';
import type { ISetPivotTableSourceRangeCommandParams, ISetPivotTableValuePositionCommandParams } from '@univerjs/sheets-pivot-table';
import type { IShowPivotTablePanelOperationParams } from '../../commands/operations/pivot-table.operation';
import { ICommandService, IUniverInstanceService, LocaleService, RichTextBuilder, UniverInstanceType } from '@univerjs/core';
import { Select } from '@univerjs/design';
import { deserializeRangeWithSheet, isReferenceString, serializeRangeToRefString, serializeRangeWithSheet } from '@univerjs/engine-formula';
import { RangeSelector } from '@univerjs/sheets-formula-ui';
import { ISheetsPivotTableService, PivotValuePosition, SetPivotTableSourceRangeCommand, SetPivotTableValuePositionCommand } from '@univerjs/sheets-pivot-table';
import { useDependency, useObservable } from '@univerjs/ui';
import { useCallback, useEffect, useMemo, useRef } from 'react';
import { PivotTableEditor } from './PivotTableEditor';

export const PivotTablePanel = (props: IShowPivotTablePanelOperationParams) => {
    const { unitId, subUnitId, pivotTableId } = props;
    const rangeSelectorInstance = useRef<IRangeSelectorInstance>(null);
    const pivotTableService = useDependency(ISheetsPivotTableService);
    const univerInstanceService = useDependency(IUniverInstanceService);
    const commandService = useDependency(ICommandService);
    const localeService = useDependency(LocaleService);
    const workbook = univerInstanceService.getUnit<Workbook>(unitId, UniverInstanceType.UNIVER_SHEET);

    const pivotTable = pivotTableService.getPivotTable(unitId, subUnitId, pivotTableId);
    const sourceRangeInfo = useObservable(pivotTable?.sourceRangeInfo$);
    const sourceSheet = sourceRangeInfo ? workbook?.getSheetBySheetId(sourceRangeInfo.subUnitId) : undefined;

    const targetCellInfo = pivotTable?.getTargetCellInfo();
    const initialValue = useMemo(() => {
        if (!sourceSheet || !sourceRangeInfo) return undefined;

        return sourceRangeInfo?.unitId === unitId
            ? serializeRangeWithSheet(sourceSheet.getName(), sourceRangeInfo.range)
            : serializeRangeToRefString({ unitId: sourceRangeInfo.unitId, sheetName: sourceRangeInfo.subUnitId, range: sourceRangeInfo.range });
    }, [sourceSheet, sourceRangeInfo, unitId]);

    const setRangeSelectorValue = useCallback((value: string | undefined) => {
        const editor = rangeSelectorInstance.current?.editor;
        if (editor) {
            const empty = RichTextBuilder.newEmptyData();
            if (typeof value === 'string') {
                editor.replaceText(value, false);
            } else {
                editor.setDocumentData(empty);
            }
        }
    }, []);

    useEffect(() => {
        // 当rangeSelector的值发生变化时更新editor的值
        setRangeSelectorValue(initialValue);
    }, [initialValue, setRangeSelectorValue]);

    const handleRangeChange = (_: IDocumentData, text: string) => {
        if (!text.trim() || !isReferenceString(text)) {
            setRangeSelectorValue(initialValue);
            return;
        }

        const result = deserializeRangeWithSheet(text);

        const newSourceSheet = workbook?.getSheetBySheetName(result.sheetName);
        const newRange = result.range;

        const newSourceRangeInfo = {
            range: newRange,
            subUnitId: newSourceSheet?.getSheetId() || sourceSheet?.getSheetId() || subUnitId,
            unitId: result.unitId || unitId,
        };

        if (
            newSourceRangeInfo.unitId === targetCellInfo?.unitId
          && newSourceRangeInfo.subUnitId === targetCellInfo?.subUnitId
          && (
              newSourceRangeInfo.range.startRow >= targetCellInfo?.row
            || newSourceRangeInfo.range.startColumn >= targetCellInfo?.col
          )
        ) {
            // 如果新源范围的起点大于等于目标单元格的行或列，则不可选择，因为可能源数据会被覆盖掉
            // TODO: 给个提示
            return;
        }

        commandService.executeCommand(SetPivotTableSourceRangeCommand.id, {
            unitId,
            subUnitId,
            pivotTableId,
            sourceRangeInfo: newSourceRangeInfo,
        } satisfies ISetPivotTableSourceRangeCommandParams);
    };

    const handleFocusChange = (isFocus: boolean) => {
        if (!sourceRangeInfo) return;
        if (isFocus) {
            rangeSelectorInstance.current?.showDialog([{
                sheetName: sourceSheet?.getName() || '',
                range: sourceRangeInfo.range,
                unitId: sourceRangeInfo.unitId,
            }]);
        }
    };

    const valuePosition = useObservable(pivotTable?.valuePosition$);
    const handleValuePositionChange = useCallback((value: string | undefined) => {
        if (!pivotTable) return;
        const newValue = Number(value) as PivotValuePosition;
        commandService.executeCommand(SetPivotTableValuePositionCommand.id, {
            unitId: pivotTable?.getTargetCellInfo().unitId,
            subUnitId: pivotTable?.getTargetCellInfo().subUnitId,
            pivotTableId: pivotTable?.getId(),
            valuePosition: newValue,
        } satisfies ISetPivotTableValuePositionCommandParams);
    }, [pivotTable, commandService.executeCommand]);

    return (
        <div className="univer-space-y-4 univer-py-4">
            <div className="univer-flex univer-flex-col univer-gap-2">
                <label
                    className={`
                      univer-text-sm univer-font-medium univer-text-gray-900
                      dark:!univer-text-white
                    `}
                >
                    {localeService.t('pivotTable.panel.sourceRangeLabel')}
                </label>
                <RangeSelector
                    selectorRef={rangeSelectorInstance}
                    unitId={unitId}
                    subUnitId={subUnitId}
                    initialValue={initialValue}
                    supportAcrossSheet
                    maxRangeCount={1}
                    isSingle
                    keepSheetReference
                    autoFocus={false}
                    onChange={handleRangeChange}
                    onFocusChange={handleFocusChange}
                />
            </div>
            <div className="univer-flex univer-flex-col univer-gap-2">
                <label
                    className={`
                      univer-text-sm univer-font-medium univer-text-gray-900
                      dark:!univer-text-white
                    `}
                >
                    {localeService.t('pivotTable.panel.fieldConfigurationLabel')}
                </label>
                {pivotTable && (
                    <PivotTableEditor pivotTable={pivotTable} />
                )}
            </div>
            <div className="univer-flex univer-flex-col univer-gap-2">
                <label
                    className={`
                      univer-text-sm univer-font-medium univer-text-gray-900
                      dark:!univer-text-white
                    `}
                >
                    {localeService.t('pivotTable.panel.valuePositionLabel')}
                </label>
                <Select
                    value={String(valuePosition)}
                    options={[
                        { label: localeService.t('pivotTable.panel.valuePositionRow'), value: String(PivotValuePosition.ROW) },
                        { label: localeService.t('pivotTable.panel.valuePositionColumn'), value: String(PivotValuePosition.COLUMN) },
                    ]}
                    onChange={handleValuePositionChange}
                />
            </div>
        </div>
    );
};

PivotTablePanel.componentKey = 'PivotTablePanel';
