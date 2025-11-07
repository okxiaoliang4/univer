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
import { CSS } from '@dnd-kit/utilities';
import { clsx } from '@univerjs/design';
import { CloseIcon, SequenceIcon } from '@univerjs/icons';
import React, { useEffect } from 'react';

export interface IFieldItemProps {
    dragOverlay?: boolean;
    color?: string;
    disabled?: boolean;
    dragging?: boolean;
    handle?: boolean;
    handleProps?: any;
    height?: number;
    index?: number;
    fadeIn?: boolean;
    transform?: Transform | null;
    listeners?: DraggableSyntheticListeners;
    sorting?: boolean;
    style?: React.CSSProperties;
    transition?: string | null;
    wrapperStyle?: React.CSSProperties;
    id: string;
    value: React.ReactNode;
    onRemove?(): void;
    renderItem?(args: {
        dragOverlay: boolean;
        dragging: boolean;
        sorting: boolean;
        index: number | undefined;
        fadeIn: boolean;
        listeners: DraggableSyntheticListeners;
        ref: React.Ref<HTMLElement>;
        style: React.CSSProperties | undefined;
        transform: IFieldItemProps['transform'];
        transition: IFieldItemProps['transition'];
        value: IFieldItemProps['value'];
    }): React.ReactElement;
}

export const FieldItem = React.memo(
    React.forwardRef<HTMLDivElement, IFieldItemProps>(
        (
            {
                color,
                dragOverlay,
                dragging,
                disabled,
                fadeIn,
                handle,
                handleProps,
                height,
                index,
                listeners,
                onRemove,
                renderItem,
                sorting,
                style,
                transition,
                transform,
                value,
                wrapperStyle,
                ...props
            },
            ref
        ) => {
            useEffect(() => {
                if (!dragOverlay) {
                    return;
                }

                document.body.style.cursor = 'grabbing';

                return () => {
                    document.body.style.cursor = '';
                };
            }, [dragOverlay]);

            return (
                <div
                    style={{
                        ...wrapperStyle,
                        transform: transform ? CSS.Translate.toString(transform) : undefined,
                    } as React.CSSProperties}
                    ref={ref}
                >
                    <div
                        className={clsx(`
                          univer-flex univer-items-center univer-justify-between univer-gap-2 univer-rounded-md
                          univer-border univer-border-solid univer-border-gray-200 univer-bg-white univer-p-2
                          univer-transition-all
                        `, [
                            dragging && 'univer-border-blue-500',
                          // sorting && 'univer-border-green-500',
                            fadeIn && 'univer-border-yellow-500',
                            dragOverlay && 'univer-border-red-500',
                            disabled && 'univer-border-gray-200',
                        ])}
                        style={style}
                        data-cypress="draggable-item"
                        {...(!handle ? listeners : undefined)}
                        {...props}
                        tabIndex={!handle ? 0 : undefined}
                    >
                        <span className="univer-flex univer-items-center univer-gap-2">
                            {handle ? <SequenceIcon {...handleProps} {...listeners} /> : null}
                            {value}
                        </span>
                        {onRemove
                            ? (
                                <CloseIcon onClick={onRemove} />
                            )
                            : null}
                    </div>
                </div>
            );
        }
    )
);
