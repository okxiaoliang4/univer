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

import type React from 'react';
import { Button, clsx } from '@univerjs/design';
import { forwardRef } from 'react';

export interface IFieldItemsContainerProps {
    className?: string;
    children: React.ReactNode;
    columns?: number;
    label?: string;
    style?: React.CSSProperties;
    hover?: boolean;
    handleProps?: React.HTMLAttributes<HTMLDivElement>;
    scrollable?: boolean;
    shadow?: boolean;
    unstyled?: boolean;
    // Add button props
    showAddButton?: boolean;
    addButtonLabel?: string;
    onAddButtonClick?: () => void;
    addButtonOverlay?: React.ReactNode;
}

export const FieldItemsContainer = forwardRef<HTMLDivElement, IFieldItemsContainerProps>(
    (
        {
            className,
            children,
            columns = 1,
            handleProps,
            hover,
            label,
            style,
            scrollable,
            shadow,
            unstyled,
            showAddButton,
            addButtonLabel = 'Add',
            onAddButtonClick,
            addButtonOverlay,
            ...props
        }: IFieldItemsContainerProps,
        ref
    ) => {
        return (
            <div
                {...props}
                className={clsx(`
                  univer-flex univer-flex-col univer-gap-2 univer-overflow-y-hidden univer-rounded-md univer-border
                  univer-border-solid univer-border-gray-200 univer-p-2
                `, [className])}
                ref={ref}
                style={{
                    ...style,
                    '--columns': columns,
                } as React.CSSProperties}
            >
                {label || showAddButton
                    ? (
                        <div className="univer-flex univer-items-center univer-justify-between univer-gap-2">
                            {label && <div className="univer-font-medium">{label}</div>}
                            {showAddButton && (
                                <div className="univer-relative">
                                    {addButtonOverlay || (
                                        <Button size="small" onClick={onAddButtonClick}>
                                            {addButtonLabel}
                                        </Button>
                                    )}
                                </div>
                            )}
                        </div>
                    )
                    : null}
                <div
                    className={clsx(`
                      univer-flex univer-h-[150px] univer-flex-col univer-gap-2 univer-overflow-scroll univer-rounded-md
                      univer-border univer-border-solid univer-border-gray-200 univer-bg-slate-50 univer-p-2
                    `, [hover && 'univer-border-primary-500'])}
                >
                    {children}
                </div>
            </div>
        );
    }
);
