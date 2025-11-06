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

import type { IShowPivotTablePanelOperationParams } from '../../commands/operations/pivot-table.operation';
import { ISheetsPivotTableService } from '@univerjs/sheets-pivot-table';
import { useDependency } from '@univerjs/ui';
import { PivotTableEditor } from './PivotTableEditor';

export const PivotTablePanel = (props: IShowPivotTablePanelOperationParams) => {
    const { unitId, subUnitId, pivotTableId } = props;
    const pivotTableService = useDependency(ISheetsPivotTableService);
    const pivotTable = pivotTableService.getPivotTable(unitId, subUnitId, pivotTableId);
    if (!pivotTable) {
        return null;
    }

    return <PivotTableEditor pivotTable={pivotTable} />;
};

PivotTablePanel.componentKey = 'PivotTablePanel';
