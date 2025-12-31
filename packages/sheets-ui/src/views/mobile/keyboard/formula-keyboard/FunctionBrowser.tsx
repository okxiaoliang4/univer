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

import { FunctionType } from '@univerjs/engine-formula';
import { IDescriptionService } from '@univerjs/sheets-formula';
import { useDependency } from '@univerjs/ui';
import { useMemo, useState } from 'react';

interface IFunctionBrowserProps {
    onSelect: (funcName: string) => void;
    onClose: () => void;
}

export function FunctionBrowser(props: IFunctionBrowserProps) {
    const { onSelect, onClose } = props;
    const descriptionService = useDependency(IDescriptionService);

    const [searchText, setSearchText] = useState('');
    const [selectedCategory, setSelectedCategory] = useState<number>(-1); // -1 for All

    const categories = useMemo(() => [
        { id: -1, name: 'All' },
        { id: FunctionType.Math, name: 'Math' },
        { id: FunctionType.Statistical, name: 'Statistical' },
        { id: FunctionType.Financial, name: 'Financial' },
        { id: FunctionType.Date, name: 'Date' },
        { id: FunctionType.Logical, name: 'Logical' },
        { id: FunctionType.Text, name: 'Text' },
        { id: FunctionType.Lookup, name: 'Lookup' },
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
        <div className="univer-fixed univer-inset-0 univer-z-[1000] univer-flex univer-flex-col univer-justify-end">
            {/* Backdrop */}
            <div
                className="univer-bg-black/40 univer-absolute univer-inset-0 univer-transition-opacity"
                onClick={onClose}
            />

            {/* Panel */}
            <div
                className={`
                  univer-relative univer-flex univer-h-[80vh] univer-w-full univer-flex-col univer-rounded-t-2xl
                  univer-bg-white univer-shadow-2xl univer-transition-transform univer-duration-300
                  dark:!univer-bg-gray-900
                `}
            >
                {/* Header/Indicator */}
                <div className="univer-flex univer-h-10 univer-shrink-0 univer-items-center univer-justify-center" onClick={onClose}>
                    <div
                        className={`
                          univer-h-1.5 univer-w-12 univer-rounded-full univer-bg-gray-300
                          dark:!univer-bg-gray-700
                        `}
                    />
                </div>

                {/* Search */}
                <div className="univer-px-4 univer-pb-2">
                    <input
                        type="text"
                        autoFocus
                        placeholder="Search functions..."
                        className={`
                          univer-w-full univer-rounded-lg univer-border-none univer-bg-gray-100 univer-px-4 univer-py-2
                          univer-text-base univer-outline-none
                          dark:!univer-bg-gray-800 dark:!univer-text-white
                        `}
                        value={searchText}
                        onChange={(e) => setSearchText(e.target.value)}
                    />
                </div>

                {/* Categories */}
                <div className="univer-flex univer-overflow-x-auto univer-px-2 univer-py-2 univer-scrollbar-none">
                    {categories.map((cat) => (
                        <button
                            key={cat.id}
                            className={`
                              univer-mr-2 univer-shrink-0 univer-rounded-full univer-px-4 univer-py-1.5 univer-text-sm
                              univer-font-medium univer-transition-colors
                              ${selectedCategory === cat.id
                            ? 'univer-bg-blue-500 univer-text-white'
                            : `
                              univer-bg-gray-100 univer-text-gray-600
                              dark:!univer-bg-gray-800 dark:!univer-text-gray-400
                            `}
                            `}
                            onClick={() => setSelectedCategory(cat.id)}
                        >
                            {cat.name}
                        </button>
                    ))}
                </div>

                {/* Function List */}
                <div className="univer-flex-1 univer-overflow-y-auto univer-px-4 univer-pb-8">
                    {filteredFunctions.length > 0
                        ? (
                            filteredFunctions.map((func) => (
                                <div
                                    key={func.name}
                                    className={`
                                      dark:hover:!univer-bg-gray-800/50
                                      univer-mb-2 univer-rounded-xl univer-border univer-border-gray-100 univer-p-4
                                      univer-transition-colors
                                      hover:univer-bg-gray-50
                                      dark:!univer-border-gray-800
                                    `}
                                    onClick={() => onSelect(func.name)}
                                >
                                    <div className="univer-flex univer-items-center univer-justify-between">
                                        <span
                                            className={`
                                              univer-text-lg univer-font-bold univer-text-blue-600
                                              dark:!univer-text-blue-400
                                            `}
                                        >
                                            {func.name}
                                        </span>
                                    </div>
                                    <p
                                        className={`
                                          univer-mt-1 univer-text-sm univer-text-gray-500
                                          dark:!univer-text-gray-400
                                        `}
                                    >
                                        {func.desc}
                                    </p>
                                </div>
                            ))
                        )
                        : (
                            <div
                                className={`
                                  univer-flex univer-h-40 univer-flex-col univer-items-center univer-justify-center
                                  univer-text-gray-400
                                `}
                            >
                                <span className="univer-text-4xl">🔍</span>
                                <p className="univer-mt-2">No functions found</p>
                            </div>
                        )}
                </div>
            </div>
        </div>
    );
}
