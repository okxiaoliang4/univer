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

import type { IShortcutItem } from '@univerjs/ui';
import { KeyCode, MetaKeys } from '@univerjs/ui';
import { OpenCreatePivotTableDialogOperation } from '../commands/operations/pivot-table.operation';

/**
 * Shortcut: Ctrl/Cmd + Shift + P - Create Pivot Table
 */
export const CreatePivotTableShortcut: IShortcutItem = {
    id: OpenCreatePivotTableDialogOperation.id,
    binding: KeyCode.P | MetaKeys.SHIFT | MetaKeys.CTRL_COMMAND,
    description: 'pivotTable.shortcut.create',
};
