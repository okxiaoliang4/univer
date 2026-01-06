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
import { Disposable, ICommandService, Inject, IUniverInstanceService, UniverInstanceType } from '@univerjs/core';
import { DataValidatorRegistryService } from '@univerjs/data-validation';
import { SheetDataValidationModel } from '@univerjs/sheets-data-validation';
import { HoverManagerService } from '@univerjs/sheets-ui';
import { filter } from 'rxjs';
import { ShowDataValidationDropdown } from '../../commands/operations/data-validation.operation';
import { DataValidationDropdownManagerService } from '../../services/dropdown-manager.service';

export class MobileDropdownTriggerController extends Disposable {
    constructor(
        @Inject(HoverManagerService) private readonly _hoverManagerService: HoverManagerService,
        @Inject(DataValidationDropdownManagerService) private readonly _dropdownManagerService: DataValidationDropdownManagerService,
        @Inject(SheetDataValidationModel) private readonly _dataValidationModel: SheetDataValidationModel,
        @Inject(DataValidatorRegistryService) private readonly _dataValidatorRegistryService: DataValidatorRegistryService,
        @IUniverInstanceService private readonly _univerInstanceService: IUniverInstanceService,
        @ICommandService private readonly _commandService: ICommandService
    ) {
        super();
        this._initClickEvent();
    }

    private _initClickEvent() {
        // On mobile, listen to cell click events and automatically open dropdown if cell has data validation
        this.disposeWithMe(
            this._hoverManagerService.currentClickedCell$.pipe(
                filter((cell) => !!cell)
            ).subscribe((cell) => {
                const { location } = cell;
                const { unitId, subUnitId, row, col } = location;

                // Check if cell has data validation dropdown
                const workbook = this._univerInstanceService.getUnit<Workbook>(unitId, UniverInstanceType.UNIVER_SHEET);
                if (!workbook) {
                    return;
                }

                const worksheet = workbook.getSheetBySheetId(subUnitId);
                if (!worksheet) {
                    return;
                }

                const rule = this._dataValidationModel.getRuleByLocation(unitId, subUnitId, row, col);
                if (!rule) {
                    return;
                }

                const validator = this._dataValidatorRegistryService.getValidatorItem(rule.type);
                if (!validator?.dropdownType) {
                    return;
                }

                // Check if dropdown is already open for this cell
                const activeDropdown = this._dropdownManagerService.activeDropdown;
                const currLoc = activeDropdown?.location;
                if (
                    currLoc &&
                    currLoc.unitId === unitId &&
                    currLoc.subUnitId === subUnitId &&
                    currLoc.row === row &&
                    currLoc.col === col
                ) {
                    // Dropdown already open for this cell, do nothing
                    return;
                }

                // Open dropdown for this cell
                this._commandService.executeCommand(ShowDataValidationDropdown.id, {
                    unitId,
                    subUnitId,
                    row,
                    column: col,
                });
            })
        );
    }
}
