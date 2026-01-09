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

import type { BooleanNumber, ICommandInfo, Workbook } from '@univerjs/core';
import type { ISetWorksheetActiveOperationParams } from '@univerjs/sheets';
import type { CSSProperties, ReactNode, Ref } from 'react';
import { ColorKit, ICommandService, ThemeService } from '@univerjs/core';
import { Button, clsx, Drawer, DrawerContent, DrawerHeader, DrawerTitle } from '@univerjs/design';
import { IncreaseIcon, LockIcon, MoreDownIcon } from '@univerjs/icons';
import {
    InsertSheetCommand,
    InsertSheetMutation,
    RangeProtectionRuleModel,
    RemoveSheetMutation,
    SetTabColorMutation,
    SetWorksheetActiveOperation,
    SetWorksheetHideMutation,
    SetWorksheetNameMutation,
    SetWorksheetOrderMutation,
    WorksheetProtectionRuleModel,
} from '@univerjs/sheets';
import { ContextMenuPosition, MobileMenu, useDependency } from '@univerjs/ui';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useActiveWorkbook } from '../../../components/hook';
import { SheetBarMenu } from '../../sheet-bar/sheet-bar-menu/SheetBarMenu';

export function MobileSheetBar() {
    const workbook = useActiveWorkbook();
    if (!workbook) {
        return null;
    }

    return <MobileSheetBarImpl workbook={workbook} />;
}

function MobileSheetBarImpl(props: { workbook: Workbook }) {
    const { workbook } = props;
    const [sheetList, setSheetList] = useState<IBaseSheetBarProps[]>([]);
    const [activeKey, setActiveKey] = useState('');
    const [drawerOpen, setDrawerOpen] = useState(false);
    const tabMapRef = useRef<Map<string, HTMLElement | null>>(new Map());

    const worksheetProtectionRuleModel = useDependency(WorksheetProtectionRuleModel);
    const rangeProtectionRuleModel = useDependency(RangeProtectionRuleModel);
    const commandService = useDependency(ICommandService);

    const updateSheetItems = useCallback(() => {
        const currentSubUnitId = workbook.getActiveSheet()!.getSheetId();
        const sheets = workbook.getSheets();
        const activeSheet = workbook.getActiveSheet();
        const sheetListItems = sheets
            .filter((sheet) => !sheet.isSheetHidden())
            .map((sheet, index) => {
                const worksheetRule = worksheetProtectionRuleModel.getRule(workbook.getUnitId(), sheet.getSheetId());
                const hasSelectionRule = rangeProtectionRuleModel.getSubunitRuleList(workbook.getUnitId(), sheet.getSheetId()).length > 0;
                const hasProtect = !!(worksheetRule?.permissionId || hasSelectionRule);
                return {
                    sheetId: sheet.getSheetId(),
                    label: sheet.getName(),
                    index,
                    selected: activeSheet === sheet,
                    color: sheet.getTabColor() ?? undefined,
                    hasProtect,
                };
            });

        setSheetList(sheetListItems);
        setActiveKey(currentSubUnitId);

        if (tabMapRef.current.has(currentSubUnitId)) {
            const element = tabMapRef.current.get(currentSubUnitId);
            if (element) {
                setTimeout(() => {
                    element.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
                }, 0);
            }
        }

        tabMapRef.current.clear();
    }, [workbook]);

    useEffect(() => updateSheetItems(), [updateSheetItems]);

    const onTabClick = useCallback((sheetId: string) => {
        if (sheetId === activeKey) {
            // If clicking the active sheet, open the drawer
            setDrawerOpen(true);
        } else {
            // If clicking an inactive sheet, switch to it
            commandService.executeCommand(SetWorksheetActiveOperation.id, {
                unitId: workbook.getUnitId(),
                subUnitId: sheetId,
            } as ISetWorksheetActiveOperationParams);
        }
    }, [activeKey, commandService, workbook]);

    const onDrawerOpenChange = useCallback((open: boolean) => {
        setDrawerOpen(open);
    }, []);

    const activeSheetName = workbook.getActiveSheet()?.getName() || '';

    useEffect(() => {
        const disposable = commandService.onCommandExecuted((commandInfo: ICommandInfo) => {
            switch (commandInfo.id) {
                case SetWorksheetHideMutation.id:
                case RemoveSheetMutation.id:
                case SetWorksheetNameMutation.id:
                case InsertSheetMutation.id:
                case SetWorksheetOrderMutation.id:
                case SetWorksheetActiveOperation.id:
                case SetTabColorMutation.id:
                    updateSheetItems();
                    // Close drawer when sheet changes
                    if (commandInfo.id === SetWorksheetActiveOperation.id) {
                        setDrawerOpen(false);
                    }
                    break;
                default:
                    break;
            }
        });

        return () => disposable.dispose();
    }, [commandService, updateSheetItems]);

    return (
        <div
            className={`
              univer-flex univer-h-8 univer-w-full univer-overflow-x-scroll univer-bg-gray-100
              dark:!univer-bg-gray-900
            `}
        >
            <SheetBarMenu size="middle" />

            <div className="univer-flex univer-h-8 univer-flex-nowrap univer-items-center">
                {sheetList.map((sheet) => (
                    <MobileSheetBarItem
                        ref={(element: HTMLDivElement | null) => {
                            tabMapRef.current.set(sheet.sheetId!, element);
                        }}
                        key={sheet.sheetId}
                        {...sheet}
                        onClick={() => onTabClick(sheet.sheetId!)}
                    />
                ))}
            </div>
            <Button
                variant="text"
                onClick={() => commandService.executeCommand(InsertSheetCommand.id)}
                className={`
                  univer-sticky univer-right-0 univer-z-10 univer-rounded-none univer-border-0 univer-border-r
                  univer-border-solid !univer-border-gray-200 univer-bg-white
                  dark:!univer-bg-gray-900
                `}
            >
                <IncreaseIcon />
            </Button>

            <Drawer open={drawerOpen} onOpenChange={onDrawerOpenChange} direction="bottom">
                <DrawerContent>
                    <DrawerHeader>
                        <DrawerTitle>{activeSheetName}</DrawerTitle>
                    </DrawerHeader>
                    <MobileMenu
                        menuType={ContextMenuPosition.FOOTER_TABS}
                        onOptionSelect={(params) => {
                            const { label: id, value, commandId } = params;
                            commandService.executeCommand(commandId ?? id as string, { value, subUnitId: activeKey });
                            setDrawerOpen(false);
                        }}
                    />
                </DrawerContent>
            </Drawer>
        </div>
    );
}

export interface IBaseSheetBarProps {
    label?: ReactNode;
    children?: any[];
    index?: number;
    color?: string;
    sheetId?: string;
    style?: CSSProperties;
    hidden?: BooleanNumber;
    selected?: boolean;
    menuOverlay?: ReactNode;
    hasProtect?: boolean;
    onClick?: () => void;
}

export function MobileSheetBarItem(props: IBaseSheetBarProps & { ref: Ref<HTMLDivElement> }) {
    const { sheetId, label, color, selected, ref, onClick, hasProtect } = props;

    const [currentSelected, setCurrentSelected] = useState(selected);

    const themeService = useDependency(ThemeService);

    useEffect(() => {
        // TODO: update too many times?
        setCurrentSelected(selected);
    }, [selected]);

    const getTextColor = (color: string) => {
        const darkTextColor = themeService.getColorFromTheme('gray.900');
        const lightTextColor = themeService.getColorFromTheme('white');
        return new ColorKit(color).isDark() ? lightTextColor : darkTextColor;
    };

    return (
        <div
            ref={ref}
            onClick={onClick}
            data-u-comp="slide-tab-item"
            key={sheetId}
            data-id={sheetId}
            className={clsx(`
              univer-box-border univer-flex univer-flex-grow univer-cursor-pointer univer-select-none univer-flex-row
              univer-items-center univer-text-xs univer-transition-[colors,box-shadow]
            `, {
                'dark:!univer-text-white': !color || (color && !getTextColor(color)),
                'univer-justify-center univer-bg-white univer-font-bold univer-text-primary-700 univer-shadow': currentSelected,
                'dark:!univer-bg-gray-700': currentSelected && !color,
                'univer-font-medium univer-text-gray-900 hover:univer-bg-gray-100': !currentSelected,
                'dark:hover:!univer-bg-gray-700': !currentSelected && !color,
            })}
            style={{
                backgroundColor: !currentSelected && color ? color : '',
                color: !currentSelected && color ? getTextColor(color) : '',
                boxShadow:
                    currentSelected && color ? `0px 0px 8px rgba(0, 0, 0, 0.08), inset 0px -2px 0px 0px ${color}` : '',
            }}
        >
            <div
                className={`
                  univer-box-border univer-flex univer-max-w-lg univer-items-center univer-gap-1 univer-overflow-hidden
                  univer-whitespace-nowrap univer-rounded univer-border-2 univer-border-solid univer-border-transparent
                  univer-px-1.5 univer-py-1
                `}
            >
                {hasProtect && (
                    <LockIcon className="univer-shrink-0" />
                )}
                {label}
                {selected && (
                    <MoreDownIcon
                        className={`
                          univer-size-3 univer-shrink-0 univer-text-primary-600
                          dark:!univer-text-white
                        `}
                    />
                )}
            </div>
        </div>
    );
}
