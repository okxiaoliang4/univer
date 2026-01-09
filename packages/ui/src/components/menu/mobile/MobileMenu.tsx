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

import type { IValueOption } from '../../../services/menu/menu';
import type { IMenuSchema } from '../../../services/menu/menu-manager.service';
import type { IBaseMenuProps } from '../desktop/Menu';
import { useMemo } from 'react';
import { IMenuManagerService } from '../../../services/menu/menu-manager.service';
import { useDependency } from '../../../utils/di';
import { MobileMenuItem } from './MobileMenuItem';

/**
 * The mobile context menu wrapper.
 */
export function MobileMenu(props: IBaseMenuProps & { onOptionSelect: (params: IValueOption) => void }) {
    const { menuType, onOptionSelect } = props;
    const menuManagerService = useDependency(IMenuManagerService);

    if (!menuType) {
        return null;
    }

    // There is no submenu on mobile devices, so if there are sub menu items, we should flat them.
    const flattedMenuItems = useMemo(() => {
        const menuItems = menuManagerService.getMenuByPositionKey(menuType);

        // Flat all menu items recursively.
        function flatMenuItems(items: IMenuSchema[]): IMenuSchema[] {
            return items.reduce((acc, item) => {
                if (item.children) {
                    return [...acc, ...flatMenuItems(item.children)];
                }
                return [...acc, item];
            }, [] as IMenuSchema[]);
        }

        return flatMenuItems(menuItems);
    }, [menuType]);

    return (
        <div
            className="univer-box-border univer-grid univer-gap-1 univer-px-2 univer-py-1"
        >
            {flattedMenuItems.map((item) => item.item && (
                <MobileMenuItem
                    key={item.key}
                    {...item.item}
                    onOptionSelect={onOptionSelect}
                />
            ))}
        </div>
    );
}
