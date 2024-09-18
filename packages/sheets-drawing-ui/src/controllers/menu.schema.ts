/**
 * Copyright 2023-present DreamNum Inc.
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

import type { MenuSchemaType } from '@univerjs/ui';
import { RibbonStartGroup } from '@univerjs/ui';
import { IMAGE_MENU_ID, ImageMenuFactory, UploadFloatImageMenuFactory } from '../views/menu/image.menu';
import { InsertFloatImageCommand } from '../commands/commands/insert-image.command';

export const menuSchema: MenuSchemaType = {
    [RibbonStartGroup.FORMULAS_INSERT]: {
        [IMAGE_MENU_ID]: {
            order: 3,
            menuItemFactory: ImageMenuFactory,
            [InsertFloatImageCommand.id]: {
                order: 0,
                menuItemFactory: UploadFloatImageMenuFactory,
            },
        },
    },
};