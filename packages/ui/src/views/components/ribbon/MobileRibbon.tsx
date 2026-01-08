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

import type { ComponentType } from 'react';
import type { Observable } from 'rxjs';
import type { RibbonType } from '../../../controllers/ui/ui.controller';
import type { IMenuSchema } from '../../../services/menu/menu-manager.service';
import { IUniverInstanceService, LocaleService } from '@univerjs/core';
import { borderBottomClassName, clsx } from '@univerjs/design';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { combineLatest } from 'rxjs';
import { IMenuManagerService } from '../../../services/menu/menu-manager.service';
import { MenuManagerPosition, RibbonPosition } from '../../../services/menu/types';
import { useDependency, useObservable } from '../../../utils/di';
import { ComponentContainer } from '../ComponentContainer';
import { MobileToolbarItem } from './MobileToolbarItem';
import { DefaultMenu } from './ribbon-menu/DefaultMenu';
import { ToolbarItem } from './ToolbarItem';

interface IRibbonProps {
    ribbonType: RibbonType;
    headerMenuComponents?: Set<ComponentType>;
    headerMenu?: boolean;
}

export function MobileRibbon(props: IRibbonProps) {
    const { ribbonType = 'default', headerMenuComponents, headerMenu = true } = props;

    const menuManagerService = useDependency(IMenuManagerService);
    const univerInstanceService = useDependency(IUniverInstanceService);
    const localeService = useDependency(LocaleService);

    const [menuChangedTimes, setMenuChangedTimes] = useState(0);

    const focusedUnit = useObservable(univerInstanceService.focused$);

    useEffect(() => {
        const subscription = menuManagerService.menuChanged$.subscribe(() => {
            setMenuChangedTimes((prev) => prev + 1);
        });

        return () => {
            subscription.unsubscribe();
        };
    }, []);

    const [ribbon, setRibbon] = useState<IMenuSchema[]>([]);
    const [activatedTab, setActivatedTab] = useState<string>(RibbonPosition.START);

    const handleSelectTab = useCallback((group: IMenuSchema) => {
        setActivatedTab(group.key);
    }, []);

    // process menu changes
    useEffect(() => {
        const ribbon = menuManagerService.getMenuByPositionKey(MenuManagerPosition.RIBBON);

        // Collect all hidden$ Observables and their corresponding paths
        const hiddenObservableMap: Observable<boolean>[] = [];
        const hiddenKeyMap: string[] = [];
        for (const group of ribbon) {
            if (group.children) {
                for (const item of group.children) {
                    if (item.children) {
                        for (const child of item.children) {
                            if (child.item?.hidden$) {
                                hiddenObservableMap.push(child.item.hidden$);
                                hiddenKeyMap.push(`${group.key}/${item.key}/${child.key}`);
                            }
                        }
                    }
                }
            }
        }

        // Only get the current value once, not continuously subscribe
        combineLatest(hiddenObservableMap)
            .subscribe((hiddenMap) => {
                const newRibbon: IMenuSchema[] = [];

                const hiddenPathMap = hiddenMap.map((hidden, index) => {
                    if (hidden) {
                        return hiddenKeyMap[index];
                    }
                    return null;
                }).filter((item) => !!item) as string[];

                for (const group of ribbon) {
                    const newGroup: IMenuSchema = { ...group, children: [] };

                    if (group.children?.length) {
                        for (const item of group.children) {
                            const newItem: IMenuSchema = { ...item, children: [] };
                            let shouldAddItem = true;

                            if (item.children?.length) {
                                for (const child of item.children) {
                                    const path = `${group.key}/${item.key}/${child.key}`;

                                    if (!hiddenPathMap.includes(path)) {
                                        newItem.children?.push(child);
                                    }
                                }

                                if (newItem.children?.every((child) => child.children?.length === 0)) {
                                    shouldAddItem = false;
                                }
                            }

                            if (shouldAddItem) {
                                newGroup.children?.push(newItem);
                            }
                        }
                    }

                    if (newGroup.children?.length && newGroup.children.every((item) => item.children?.length)) {
                        newRibbon.push(newGroup);
                    }
                }

                if (ribbonType === 'simple') {
                    const simpleRibbon: IMenuSchema[] = [{ key: RibbonPosition.START, children: [], order: 0 }];
                    newRibbon.forEach((group) => {
                        group.children?.forEach((item) => {
                            simpleRibbon[0].children?.push(item);
                        });
                    });

                    setRibbon(simpleRibbon);
                } else {
                    setRibbon(newRibbon);
                }
            })
            .unsubscribe();
    }, [menuChangedTimes, focusedUnit, ribbonType]);

    const activeGroups = useMemo(() => {
        return ribbon.find((group) => group.key === activatedTab)?.children ?? [];
    }, [ribbon, activatedTab]);

    // Fetch and process subMenu items
    const [filteredSubMenuItems, setFilteredSubMenuItems] = useState<IMenuSchema[]>([]);

    useEffect(() => {
        const subMenu = menuManagerService.getMenuByPositionKey(RibbonPosition.SUBMENU);

        // Collect hidden$ observables for subMenu items
        const hiddenObservableMap: Observable<boolean>[] = [];
        const hiddenKeyMap: string[] = [];
        for (const item of subMenu) {
            if (item.item?.hidden$) {
                hiddenObservableMap.push(item.item.hidden$);
                hiddenKeyMap.push(item.key);
            }
        }

        if (hiddenObservableMap.length === 0) {
            setFilteredSubMenuItems(subMenu);
            return;
        }

        const subscription = combineLatest(hiddenObservableMap)
            .subscribe((hiddenMap) => {
                const hiddenKeySet = new Set(
                    hiddenMap.map((hidden, index) => hidden ? hiddenKeyMap[index] : null).filter((key) => key !== null)
                );

                const filtered = subMenu.filter((item) => !hiddenKeySet.has(item.key));
                setFilteredSubMenuItems(filtered);
            });

        // Set initial value
        setFilteredSubMenuItems(subMenu);

        return () => {
            subscription.unsubscribe();
        };
    }, [menuChangedTimes, menuManagerService]);

    return (
        <>
            <div
                data-u-comp="ribbon-header-menu"
                className={clsx('univer-relative univer-select-none', {
                    'univer-h-9': (headerMenuComponents && headerMenuComponents.size > 0),
                })}
            >
                {headerMenu && (headerMenuComponents && headerMenuComponents.size > 0) && (
                    <div
                        className={`
                          univer-absolute univer-right-2 univer-top-0 univer-flex univer-h-full univer-items-center
                          univer-gap-2
                          [&>*]:univer-inline-flex [&>*]:univer-h-6 [&>*]:univer-items-center [&>*]:univer-rounded
                          [&>*]:univer-px-1 [&>*]:univer-transition-colors
                          hover:[&>*]:univer-bg-gray-100
                        `}
                    >
                        <ComponentContainer components={headerMenuComponents} />
                    </div>
                )}
            </div>

            <div
                className={clsx('univer-box-border univer-flex univer-flex-col', borderBottomClassName)}
            >
                <div className="univer-flex univer-flex-none univer-items-center univer-justify-between univer-px-3">
                    {ribbonType === 'default' && ribbon.length > 1 && (
                        <DefaultMenu
                            ribbon={ribbon}
                            activatedTab={activatedTab}
                            onSelectTab={handleSelectTab}
                        />
                    )}
                    <div className="univer-flex univer-items-center univer-gap-2">
                        {filteredSubMenuItems.map((item) => (
                            item.item && (
                                <ToolbarItem key={item.key} {...item.item} />
                            )
                        ))}
                    </div>
                </div>

                <div
                    data-u-comp="ribbon-toolbar"
                    className="univer-flex univer-h-[30vh] univer-flex-col univer-overflow-y-auto univer-py-2"
                    role="toolbar"
                    aria-label={localeService.t(activatedTab)}
                >
                    {activeGroups.map((groupItem, groupIndex) => {
                        if (!groupItem.children?.length && !groupItem.item) {
                            return null;
                        }

                        return (
                            <div key={groupItem.key}>
                                {groupIndex > 0 && (
                                    <div
                                        className={`
                                          univer-my-2 univer-h-px univer-bg-gray-200
                                          dark:!univer-bg-gray-700
                                        `}
                                    />
                                )}
                                <div className="univer-flex univer-flex-col univer-gap-2 univer-px-2">
                                    {groupItem.children && groupItem.children.map((child) => (
                                        child.item && (
                                            <MobileToolbarItem key={child.key} {...child.item} />
                                        )
                                    ))}
                                </div>
                            </div>
                        );
                    })}
                </div>
            </div>
        </>
    );
}
