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

import type { IAccessor, ICommand } from '@univerjs/core';
import { CommandType, LocaleService } from '@univerjs/core';

import { ISidebarService } from '@univerjs/ui';
import { ISheetBarService } from '../../services/sheet-bar/sheet-bar.service';
import { RenameInput } from '../../views/mobile/sheet-bar/RenameInput';

interface IRenameSheetOperationParams {
    subUnitId: string;
}

export const RenameSheetOperation: ICommand = {
    id: 'sheet.operation.rename-sheet',
    type: CommandType.OPERATION,
    handler: async (accessor: IAccessor, params?: IRenameSheetOperationParams) => {
        const sheetBarService = accessor.get(ISheetBarService);
        if (params) {
            sheetBarService.setRenameId(params.subUnitId);
        }
        return true;
    },
};

export const RENAME_INPUT_DIALOG_ID = 'rename-input-dialog';
export const RenameSheetMobileOperation: ICommand = {
    id: 'sheet.operation.mobile-rename-sheet',
    type: CommandType.OPERATION,
    handler: async (accessor: IAccessor, params?: IRenameSheetOperationParams) => {
        const sidebarService = accessor.get(ISidebarService);
        const localeService = accessor.get(LocaleService);
        if (params) {
            sidebarService.open({
                id: RENAME_INPUT_DIALOG_ID,
                header: {
                    title: localeService.t('sheetConfig.rename'),
                },
                children: {
                    label: {
                        name: RenameInput.componentKey,
                        props: {
                            subUnitId: params.subUnitId,
                        },
                    },
                },
            });
        }
        return true;
    },
};
