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

import type { IRange } from '@univerjs/core';
import type { IPivotTableSelectionInfo } from '../../commands/operations/pivot-table.operation';
import { IUniverInstanceService, LocaleService, Rectangle } from '@univerjs/core';
import { Button, Radio, RadioGroup } from '@univerjs/design';
import { deserializeRangeWithSheet, serializeRange } from '@univerjs/engine-formula';
import { getSheetCommandTarget } from '@univerjs/sheets';
import { RangeSelector } from '@univerjs/sheets-formula-ui';
import { useDependency } from '@univerjs/ui';
import { useState } from 'react';

export const CreatePivotTableDialog = (props: IPivotTableSelectionInfo & {
    onConfirm: (info: IPivotTableSelectionInfo) => void;
    onCancel: () => void;
}) => {
    const { unitId, subUnitId, sourceRange, targetRange, targetRangeType, onCancel, onConfirm } = props;

    const [selectedSourceRange, setSelectedSourceRange] = useState(sourceRange);
    const [selectedTargetRange, setSelectedTargetRange] = useState(targetRange);
    const [selectedTargetRangeType, setSelectedTargetRangeType] = useState<'new' | 'existing'>(targetRangeType || 'new');
    const [sourceRangeError, setSourceRangeError] = useState('');
    const [targetRangeError, setTargetRangeError] = useState('');
    const localeService = useDependency(LocaleService);
    const univerInstanceService = useDependency(IUniverInstanceService);

    const validateSourceRange = (range: IRange): string => {
        const { startRow, endRow, startColumn, endColumn } = range;

        // Check if it's a single cell
        if (startRow === endRow && startColumn === endColumn) {
            return localeService.t('pivotTable.dialog.sourceRangeSingleCellError');
        }

        // Check if it has at least 2 rows (header + data)
        if (startRow === endRow) {
            return localeService.t('pivotTable.dialog.sourceRangeSingleRowError');
        }

        // Check for merged cells
        const target = getSheetCommandTarget(univerInstanceService, { unitId, subUnitId });
        if (!target) {
            return localeService.t('pivotTable.dialog.invalidWorksheet');
        }
        const worksheet = target.worksheet;
        const merges = worksheet.getMergeData();
        const hasOverlapWithMerge = merges.some((merge) => {
            return Rectangle.intersects(range, merge);
        });

        if (hasOverlapWithMerge) {
            return localeService.t('pivotTable.dialog.sourceRangeWithMergeError');
        }

        return '';
    };

    const validateTargetRange = (range: IRange): string => {
        // Target range should be a single cell
        const { startRow, endRow, startColumn, endColumn } = range;
        if (startRow !== endRow || startColumn !== endColumn) {
            return localeService.t('pivotTable.dialog.targetRangeSingleCellError');
        }

        return '';
    };

    return (
        <div className="univer-space-y-4">
            <div>
                <div className="univer-mb-2 univer-text-sm univer-font-medium">
                    {localeService.t('pivotTable.dialog.sourceRangeLabel')}
                </div>
                <RangeSelector
                    maxRangeCount={1}
                    isSingle
                    unitId={unitId}
                    subUnitId={subUnitId}
                    initialValue={serializeRange(sourceRange)}
                    onChange={(_, text) => {
                        const newRange = deserializeRangeWithSheet(text).range;
                        const error = validateSourceRange(newRange);
                        setSourceRangeError(error);
                        if (!error) {
                            setSelectedSourceRange(newRange);
                        }
                    }}
                    supportAcrossSheet={false}
                />
                {sourceRangeError && (
                    <div className="univer-mt-1 univer-text-xs univer-text-red-500">
                        {sourceRangeError}
                    </div>
                )}
            </div>

            <div>
                <div className="univer-mb-2 univer-text-sm univer-font-medium">
                    {localeService.t('pivotTable.dialog.targetRangeLabel')}
                </div>
                <RadioGroup
                    value={selectedTargetRangeType}
                    onChange={(value) => {
                        setSelectedTargetRangeType(value as 'new' | 'existing');
                        if (value === 'new') {
                            setTargetRangeError('');
                        }
                    }}
                    direction="vertical"
                >
                    <Radio value="new">
                        {localeService.t('pivotTable.dialog.targetRangeTypeNew')}
                    </Radio>
                    <Radio value="existing">
                        {localeService.t('pivotTable.dialog.targetRangeTypeExisting')}
                    </Radio>
                </RadioGroup>
                {selectedTargetRangeType === 'existing' && (
                    <div className="univer-mt-3">
                        <RangeSelector
                            unitId={unitId}
                            subUnitId={subUnitId}
                            initialValue={targetRange ? serializeRange(targetRange) : ''}
                            supportAcrossSheet
                            maxRangeCount={1}
                            isSingle
                            autoFocus={true}
                            onChange={(_, text) => {
                                const newRange = deserializeRangeWithSheet(text).range;
                                const error = validateTargetRange(newRange);
                                setTargetRangeError(error);
                                if (!error) {
                                    setSelectedTargetRange(newRange);
                                }
                            }}
                        />
                        {targetRangeError && (
                            <div className="univer-mt-1 univer-text-xs univer-text-red-500">
                                {targetRangeError}
                            </div>
                        )}
                    </div>
                )}
            </div>

            <div className="univer-flex univer-justify-end univer-gap-2">
                <Button onClick={onCancel}>
                    {localeService.t('pivotTable.dialog.cancel')}
                </Button>
                <Button
                    variant="primary"
                    onClick={() => {
                        if (sourceRangeError || (selectedTargetRangeType === 'existing' && (targetRangeError || !selectedTargetRange))) {
                            return;
                        }
                        onConfirm({
                            unitId,
                            subUnitId,
                            sourceRange: selectedSourceRange,
                            targetRange: selectedTargetRangeType === 'existing' ? selectedTargetRange : undefined,
                            targetRangeType: selectedTargetRangeType,
                        });
                    }}
                    disabled={!!sourceRangeError || (selectedTargetRangeType === 'existing' && (!!targetRangeError || !selectedTargetRange))}
                >
                    {localeService.t('pivotTable.dialog.confirm')}
                </Button>
            </div>
        </div>
    );
};
