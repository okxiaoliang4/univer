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

import type { IUpdatePivotTableFieldsCommandParams } from '@univerjs/sheets-pivot-table';
import { Disposable, ICommandService, Inject } from '@univerjs/core';
import { UpdatePivotTableFieldsCommand } from '@univerjs/sheets-pivot-table';
import { IntelligentFieldPlacementService } from '../services/intelligent-field-placement.service';

export class PivotTableController extends Disposable {
    constructor(
        @ICommandService private readonly _commandService: ICommandService,
        @Inject(IntelligentFieldPlacementService) private readonly _intelligentPlacementService: IntelligentFieldPlacementService
    ) {
        super();
        this._initCommands();
    }

    private _initCommands(): void {
        this.disposeWithMe(
            this._commandService.onCommandExecuted((command) => {
                if (command.id === UpdatePivotTableFieldsCommand.id) {
                    const params = command.params as IUpdatePivotTableFieldsCommandParams;
                    this._intelligentPlacementService.updatePreference(params.fieldsConfig);
                }
            })
        );
    }
}
