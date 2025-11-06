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

import type { IMenuButtonItem, MenuSchemaType } from '@univerjs/ui';
import { MenuItemType, RibbonDataGroup } from '@univerjs/ui';
import { Observable } from 'rxjs';
import { OpenCreatePivotTableDialogOperation } from '../commands/operations/pivot-table.operation';

export const PIVOT_TABLE_MENU_ID = 'pivotTable.menu.insert';

export const menuSchema: MenuSchemaType = {
    [RibbonDataGroup.OTHERS]: {
        [PIVOT_TABLE_MENU_ID]: {
            order: 0,
            menuItemFactory: InsertPivotTableMenuItemFactory,
        },
    },
};

/**
 * Menu item: Insert > Pivot Table
 */
export function InsertPivotTableMenuItemFactory(): IMenuButtonItem {
    return {
        id: OpenCreatePivotTableDialogOperation.id,
        type: MenuItemType.BUTTON,
        title: 'pivotTable.menu.insert',
        icon: 'CopyDoubleIcon',
        disabled$: new Observable((subscriber) => {
            subscriber.next(false);
        }),
    };
}
