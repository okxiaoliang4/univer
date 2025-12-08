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

import type { DraggableSyntheticListeners } from '@dnd-kit/core';
import type { Transform } from '@dnd-kit/utilities';
import type { IPivotField } from '@univerjs/sheets-pivot-table';
import { CSS } from '@dnd-kit/utilities';
import { Checkbox, clsx } from '@univerjs/design';
import { CloseIcon, SequenceIcon } from '@univerjs/icons';
import { forwardRef } from 'react';

export interface IFieldRenderProps {
    field: IPivotField;
    renderFooter?: () => React.ReactNode;
    onRemove?: () => void;
    // Checkbox props for source fields
    showCheckbox?: boolean;
    checkboxChecked?: boolean;
    onCheckboxChange?: (checked: boolean) => void;

    dragOverlay: boolean;
    dragging: boolean;
    sorting: boolean;
    index: number | undefined;
    fadeIn: boolean;
    listeners: DraggableSyntheticListeners;
    style: React.CSSProperties | undefined;
    transform: Transform | null;
    transition: string | null;
    value: string;
}

export const FieldRender = forwardRef<HTMLDivElement, IFieldRenderProps>((props, ref) => {
    const {
        dragging,
        fadeIn,
        dragOverlay,
        sorting,
        style,
        transform,
        listeners,
        value,
        onRemove,
        renderFooter,
        showCheckbox,
        checkboxChecked,
        onCheckboxChange,
    } = props;

    const handleCheckboxChange = (checked: boolean) => {
        onCheckboxChange?.(checked);
    };

    return (
        <div
            className={`
              univer-flex univer-flex-col univer-gap-4 univer-rounded-md univer-border univer-border-solid
              univer-border-gray-200 univer-bg-white univer-p-2
            `}
            style={{
                transform: transform ? CSS.Translate.toString(transform) : undefined,
            } as React.CSSProperties}
            ref={ref as React.Ref<HTMLDivElement>}
        >
            <div
                className={clsx(`
                  univer-flex univer-items-center univer-justify-between univer-gap-2 univer-transition-all
                `, [
                    dragging && 'univer-border-blue-500',
                    sorting && 'univer-border-green-500',
                    fadeIn && 'univer-border-yellow-500',
                    dragOverlay && 'univer-border-red-500',
                ])}
                style={style}
                data-cypress="draggable-item"
            >
                <span className="univer-flex univer-items-center univer-gap-2 univer-font-bold">
                    {showCheckbox && (
                        <Checkbox
                            checked={checkboxChecked ?? false}
                            onChange={handleCheckboxChange}
                            onClick={(e) => {
                                e.stopPropagation();
                            }}
                        />
                    )}
                    <SequenceIcon className="univer-cursor-move" {...listeners} />
                    {value}
                </span>
                {onRemove
                    ? (
                        <CloseIcon className="univer-cursor-pointer" onClick={onRemove} />
                    )
                    : null}
            </div>
            {renderFooter?.()}
        </div>
    );
});
