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

import type { IDisplayMenuItem, IMenuItem, IMenuSelectorItem, IValueOption } from '../../../services/menu/menu';
import type { IMenuSchema } from '../../../services/menu/menu-manager.service';
import { ICommandService, LocaleService } from '@univerjs/core';
import { clsx, Drawer, DrawerContent, DrawerHeader, DrawerTitle } from '@univerjs/design';
import { MoreRightIcon } from '@univerjs/icons';
import { useEffect, useMemo, useState } from 'react';
import { combineLatest, isObservable, Observable, of } from 'rxjs';
import { ComponentManager } from '../../../common/component-manager';
import { CustomLabel } from '../../../components/custom-label/CustomLabel';
import { t } from '../../../components/hooks/locale';
import { ILayoutService } from '../../../services/layout/layout.service';
import { MenuItemType } from '../../../services/menu/menu';
import { IMenuManagerService } from '../../../services/menu/menu-manager.service';
import { useDependency, useObservable } from '../../../utils/di';
import { useToolbarItemStatus } from '../../../views/components/ribbon/hook';

export function MobileMenuItem(props: IDisplayMenuItem<IMenuItem>) {
    const localeService = useDependency(LocaleService);
    const commandService = useDependency(ICommandService);
    const layoutService = useDependency(ILayoutService);
    const componentManager = useDependency(ComponentManager);
    const menuManagerService = useDependency(IMenuManagerService);

    const { value, hidden, disabled, activated } = useToolbarItemStatus(props);

    const executeCommand = (commandId: string, params?: Record<string, unknown>) => {
        layoutService.focus();
        commandService.executeCommand(commandId, params);
    };

    const { title, tooltip, icon, label, id, commandId, type, params } = props;

    const { selections } = props as IDisplayMenuItem<IMenuSelectorItem>;
    const selections$ = useMemo(() => {
        if (isObservable(selections)) {
            return selections;
        } else {
            return new Observable<typeof selections>((subscribe) => {
                subscribe.next(selections);
            });
        }
    }, [selections]);
    const options = useObservable(selections$) as IValueOption[];

    const icon$ = useMemo(() => {
        if (isObservable(icon)) {
            return icon;
        } else {
            return new Observable<typeof icon>((subscribe) => {
                const v = options?.find((o) => o.value === value)?.icon ?? icon;
                subscribe.next(v);
            });
        }
    }, [icon, options, value]);

    const iconToDisplay = useObservable(icon$, undefined, true);

    const [drawerOpen, setDrawerOpen] = useState(false);
    const [hiddenStates, setHiddenStates] = useState<Record<string, boolean>>({});

    // Get submenu items for SUBITEMS type
    const subMenuItems = useMemo(() => {
        if (type === MenuItemType.SUBITEMS && id) {
            return menuManagerService.getMenuByPositionKey(id);
        }
        return [];
    }, [type, id, menuManagerService]);

    // Track hidden states for submenu items
    useEffect(() => {
        if (type !== MenuItemType.SUBITEMS || !subMenuItems.length) return;

        const subscriptions: Array<{ unsubscribe: () => void }> = [];

        subMenuItems.forEach((item) => {
            if (!item.children) {
                if (item.item?.hidden$) {
                    const sub = item.item.hidden$.subscribe((hidden) => {
                        setHiddenStates((prev) => ({
                            ...prev,
                            [item.key]: hidden,
                        }));
                    });
                    subscriptions.push(sub);
                }
            } else {
                const hiddenObservables = item.children.map((subItem) => subItem.item?.hidden$ ?? of(false));

                const sub = combineLatest(hiddenObservables).subscribe((hiddenValues) => {
                    const isAllHidden = hiddenValues.every((hidden) => hidden === true);
                    setHiddenStates((prev) => ({
                        ...prev,
                        [item.key]: isAllHidden,
                    }));
                });

                subscriptions.push(sub);
            }
        });

        return () => {
            subscriptions.forEach((sub) => sub?.unsubscribe());
            setHiddenStates({});
        };
    }, [type, subMenuItems]);

    const filteredSubMenuItems = useMemo(() => {
        return subMenuItems.filter((item) => {
            if (!item.children) {
                return !hiddenStates[item.key];
            }

            const itemKey = item.key?.toString() || '';
            return !hiddenStates[itemKey];
        });
    }, [subMenuItems, hiddenStates]);

    function renderSelectorType(menuType: MenuItemType) {
        const selectionsCommandId = (props as IDisplayMenuItem<IMenuSelectorItem>).selectionsCommandId;
        const sId = selectionsCommandId ?? commandId ?? id;

        function handleSelect(option: IValueOption) {
            if (disabled) return;

            let commandId = sId;
            if (option.id) {
                commandId = option.id;
            } else if (option.commandId) {
                commandId = option.commandId;
            }

            executeCommand(commandId, { value: option.value });
            setDrawerOpen(false);
        }

        function handleSelectionsValueChange(value: string | number) {
            if (disabled) return;
            executeCommand(sId, { value });
        }

        function handleClick() {
            if (disabled) return;
            setDrawerOpen(true);
        }

        // Render drawer content
        function renderDrawerContent() {
            const items: Array<{ option?: IValueOption; menuItem?: IMenuSchema }> = [];

            // Add options if available
            if (options?.length) {
                options.forEach((option) => {
                    items.push({ option });
                });
            }

            // Add submenu items for SUBITEMS type
            if (menuType === MenuItemType.SUBITEMS) {
                filteredSubMenuItems.forEach((menuItem) => {
                    if (menuItem.item) {
                        items.push({ menuItem });
                    }
                });
            }

            return (
                <div className="univer-flex univer-flex-col univer-gap-2 univer-p-4">
                    {items.map((item) => {
                        if (item.option) {
                            const option = item.option;

                            return (
                                <div
                                    key={option.value}
                                    onClick={() => handleSelect(option)}
                                    className={clsx(`
                                      univer-relative univer-flex univer-cursor-pointer univer-gap-2 univer-rounded-md
                                      univer-p-2 univer-transition-colors
                                    `)}
                                >
                                    <div className="univer-flex univer-flex-1 univer-items-center univer-gap-2">
                                        <CustomLabel
                                            icon={option.icon}
                                            value={option.value}
                                            label={option.label}
                                            onChange={handleSelectionsValueChange}
                                        />
                                    </div>
                                </div>
                            );
                        } else if (item.menuItem?.item) {
                            const menuItem = item.menuItem.item;
                            const menuItemId = menuItem.commandId ?? menuItem.id;

                            return (
                                <div
                                    key={menuItemId}
                                    onClick={() => {
                                        executeCommand(menuItemId, {});
                                        setDrawerOpen(false);
                                    }}
                                    className={clsx(`
                                      univer-flex univer-h-4 univer-w-full univer-cursor-pointer univer-items-center
                                      univer-gap-3 univer-py-2 univer-transition-colors
                                    `)}
                                >
                                    <div className="univer-flex univer-flex-1 univer-items-center univer-gap-2">
                                        <CustomLabel
                                            title={menuItem.title}
                                            icon={menuItem.icon}
                                            label={menuItem.label}
                                            onChange={handleSelectionsValueChange}
                                        />
                                    </div>
                                </div>
                            );
                        }
                        return null;
                    })}
                </div>
            );
        }

        return (
            <>
                <div
                    data-u-command={id}
                    data-disabled={disabled}
                    onClick={handleClick}
                    className={clsx(`
                      univer-flex univer-h-4 univer-cursor-pointer univer-items-center univer-gap-3 univer-rounded-md
                      univer-px-4 univer-py-2 univer-transition-colors
                      active:univer-bg-gray-100
                      dark:active:!univer-bg-gray-700
                    `, {
                        'univer-text-gray-900 dark:!univer-text-white': !disabled,
                        'univer-pointer-events-none univer-cursor-not-allowed univer-text-gray-300 dark:!univer-text-gray-600': disabled,
                        'univer-bg-gray-100 dark:!univer-bg-gray-700': activated,
                    })}
                >
                    <div className="univer-flex univer-flex-1 univer-items-center univer-gap-2">
                        <CustomLabel
                            title={title ?? tooltip}
                            value={value}
                            label={label}
                            icon={iconToDisplay}
                            onChange={handleSelectionsValueChange}
                        />
                    </div>
                    <div className="univer-flex univer-shrink-0 univer-items-center">
                        <MoreRightIcon className="univer-text-gray-400" />
                    </div>
                </div>

                {(menuType === MenuItemType.SELECTOR || menuType === MenuItemType.SUBITEMS || menuType === MenuItemType.BUTTON_SELECTOR) && (
                    <Drawer
                        open={drawerOpen}
                        onOpenChange={setDrawerOpen}
                        direction="bottom"
                    >
                        <DrawerContent>
                            <DrawerHeader>
                                <DrawerTitle>{t(localeService, tooltip ?? '')}</DrawerTitle>
                            </DrawerHeader>
                            {renderDrawerContent()}
                        </DrawerContent>
                    </Drawer>
                )}
            </>
        );
    }

    function renderButtonType() {
        const isCustomComponent = componentManager.get(typeof label === 'string' ? label : label?.name ?? '');

        const commandValue = value ?? typeof params === 'function' ? params() : params;

        const displayTitle = tooltip ? localeService.t(tooltip) : (title ? localeService.t(title) : undefined);

        return (
            <div
                data-u-command={id}
                onClick={() => executeCommand(props.commandId ?? props.id, commandValue)}
                onDoubleClick={() => props.subId && executeCommand(props.subId)}
                className={clsx(`
                  univer-flex univer-h-4 univer-cursor-pointer univer-items-center univer-gap-3 univer-rounded-md
                  univer-px-4 univer-py-2 univer-transition-colors
                  active:univer-bg-gray-100
                  dark:active:!univer-bg-gray-700
                `, {
                    'univer-text-gray-900 dark:!univer-text-white': !disabled,
                    'univer-pointer-events-none univer-cursor-not-allowed univer-text-gray-300 dark:!univer-text-gray-600': disabled,
                    'univer-bg-gray-100 dark:!univer-bg-gray-700': activated,
                })}
            >
                <div className="univer-flex univer-flex-1 univer-items-center univer-gap-2">
                    {isCustomComponent
                        ? (
                            <CustomLabel icon={icon} title={displayTitle} value={value} label={label} />
                        )
                        : (
                            <CustomLabel icon={icon} title={displayTitle} />
                        )}
                </div>
            </div>
        );
    }

    function renderItem() {
        switch (type) {
            case MenuItemType.BUTTON_SELECTOR:
            case MenuItemType.SELECTOR:
            case MenuItemType.SUBITEMS:
                return renderSelectorType(type);
            case MenuItemType.BUTTON:
            default:
                return renderButtonType();
        }
    }

    return !hidden && renderItem();
}
