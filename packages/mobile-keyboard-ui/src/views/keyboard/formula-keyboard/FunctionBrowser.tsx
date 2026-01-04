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

import type { IDrawerProps } from '@univerjs/design';
import { Drawer, DrawerContent, DrawerHeader, DrawerTitle, Input, scrollbarClassName, Segmented } from '@univerjs/design';
import { FunctionType } from '@univerjs/engine-formula';
import { IDescriptionService } from '@univerjs/sheets-formula';
import { useDependency } from '@univerjs/ui';
import { useMemo, useState } from 'react';

export type IFunctionBrowserProps = IDrawerProps & {
    onSelect: (funcName: string) => void;
};
export function FunctionBrowser(props: IFunctionBrowserProps) {
    const { onSelect, ...rest } = props;
    const descriptionService = useDependency(IDescriptionService);

    const [searchText, setSearchText] = useState('');
    const [selectedCategory, setSelectedCategory] = useState<number>(-1); // -1 for All

    const categories = useMemo(() => [
        { id: -1, name: 'All', label: 'All' },
        { id: FunctionType.Math, name: 'Math', label: 'Math' },
        { id: FunctionType.Statistical, name: 'Statistical', label: 'Statistical' },
        { id: FunctionType.Financial, name: 'Financial', label: 'Financial' },
        { id: FunctionType.Date, name: 'Date', label: 'Date' },
        { id: FunctionType.Logical, name: 'Logical', label: 'Logical' },
        { id: FunctionType.Text, name: 'Text', label: 'Text' },
        { id: FunctionType.Lookup, name: 'Lookup', label: 'Lookup' },
    ], []);

    const filteredFunctions = useMemo(() => {
        let list = descriptionService.getSearchListByType(selectedCategory);
        if (searchText.trim()) {
            const search = searchText.toLowerCase();
            list = list.filter((f) =>
                f.name.toLowerCase().includes(search) ||
                f.desc.toLowerCase().includes(search)
            );
        }
        return list;
    }, [descriptionService, selectedCategory, searchText]);

    return (
        <Drawer {...rest} modal={false} shouldScaleBackground>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle className="univer-space-y-4">
                        {/* Search */}
                        <div className="univer-px-1">
                            <Input
                                placeholder="Search functions..."
                                size="middle"
                                value={searchText}
                                onChange={setSearchText}
                                allowClear
                                className="univer-w-full"
                            />
                        </div>

                        {/* Categories */}
                        <div
                            className={`
                              univer-overflow-x-auto univer-px-1
                              ${scrollbarClassName}
                            `}
                        >
                            <Segmented
                                items={categories.map((cat) => ({ label: cat.label, value: cat.id }))}
                                value={selectedCategory}
                                onChange={(value) => setSelectedCategory(value as number)}
                                className="univer-inline-flex univer-w-max"
                            />
                        </div>
                    </DrawerTitle>
                </DrawerHeader>

                {/* Function List */}
                <div
                    className={`
                      univer-flex-1 univer-overflow-y-auto univer-px-4 univer-pb-6
                      ${scrollbarClassName}
                    `}
                >
                    {filteredFunctions.length > 0
                        ? (
                            <div className="univer-space-y-2">
                                {filteredFunctions.map((func) => (
                                    <button
                                        key={func.name}
                                        type="button"
                                        className={`
                                          hover:univer-bg-primary-50/50 hover:univer-border-primary-300
                                          hover:univer-shadow-sm
                                          dark:hover:!univer-bg-gray-700/50 dark:hover:!univer-border-primary-600
                                          univer-group univer-w-full univer-rounded-lg univer-border
                                          univer-border-gray-200 univer-bg-white univer-p-4 univer-text-left
                                          univer-transition-all
                                          active:univer-scale-[0.98]
                                          dark:!univer-border-gray-700 dark:!univer-bg-gray-800
                                        `}
                                        onClick={() => onSelect(func.name)}
                                    >
                                        <div className="univer-flex univer-items-start univer-justify-between">
                                            <div className="univer-flex-1">
                                                <div
                                                    className={`
                                                      univer-text-base univer-font-semibold univer-text-primary-600
                                                      group-hover:univer-text-primary-700
                                                      dark:!univer-text-primary-400
                                                      dark:group-hover:!univer-text-primary-300
                                                    `}
                                                >
                                                    {func.name}
                                                </div>
                                                <p
                                                    className={`
                                                      univer-mt-1.5 univer-text-sm univer-leading-relaxed
                                                      univer-text-gray-600
                                                      dark:!univer-text-gray-300
                                                    `}
                                                >
                                                    {func.desc}
                                                </p>
                                            </div>
                                        </div>
                                    </button>
                                ))}
                            </div>
                        )
                        : (
                            <div
                                className={`
                                  univer-flex univer-h-64 univer-flex-col univer-items-center univer-justify-center
                                  univer-text-gray-400
                                `}
                            >
                                <div className="univer-mb-3 univer-text-5xl">🔍</div>
                                <p className="univer-text-base univer-font-medium">No functions found</p>
                                <p
                                    className={`
                                      univer-mt-1 univer-text-sm univer-text-gray-500
                                      dark:!univer-text-gray-500
                                    `}
                                >
                                    Try a different search term or category
                                </p>
                            </div>
                        )}
                </div>
            </DrawerContent>
        </Drawer>
    );
}
