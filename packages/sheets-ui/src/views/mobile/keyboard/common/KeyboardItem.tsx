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

import type { ButtonHTMLAttributes, ReactNode } from 'react';
import { clsx } from '@univerjs/design';
import { forwardRef } from 'react';

export interface KeyboardItemProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'className'> {
    /** Button variant */
    variant?: 'default' | 'primary' | 'danger' | 'success' | 'blue';
    /** Additional className for custom styling */
    className?: string;
    /** Button content */
    children: ReactNode;
    /** Span columns in grid layout */
    colSpan?: number;
    /** Span rows in grid layout */
    rowSpan?: number;
}

export const KeyboardItem = forwardRef<HTMLButtonElement, KeyboardItemProps>(
    ({ variant = 'default', className, children, colSpan, rowSpan, ...rest }, ref) => {
        const variantStyles = {
            default: `
                univer-bg-white univer-text-gray-700
                hover:univer-bg-gray-100
                dark:!univer-bg-gray-700 dark:!univer-text-gray-300
                dark:hover:!univer-bg-gray-600
            `,
            primary: `
                univer-bg-blue-500 univer-text-white
                hover:univer-bg-blue-600
                dark:!univer-bg-blue-600 dark:hover:!univer-bg-blue-700
            `,
            danger: `
                univer-bg-red-500 univer-text-white
                hover:univer-bg-red-600
                dark:!univer-bg-red-600 dark:hover:!univer-bg-red-700
            `,
            success: `
                univer-bg-green-500 univer-text-white
                hover:univer-bg-green-600
                dark:!univer-bg-green-600 dark:hover:!univer-bg-green-700
            `,
            blue: `
                univer-bg-blue-500 univer-text-white
                hover:univer-bg-blue-600
                dark:!univer-bg-blue-600 dark:hover:!univer-bg-blue-700
            `,
        };

        return (
            <button
                ref={ref}
                type="button"
                className={clsx(
                    // Base styles
                    'univer-rounded-md univer-shadow-sm',
                    'univer-transition-all univer-duration-150',
                    'univer-font-medium',
                    'focus:univer-outline-none focus:univer-ring-2 focus:univer-ring-blue-300',
                    'active:univer-scale-95',
                    'univer-border-none',
                    // Variant styles
                    variantStyles[variant],
                    // Grid span styles
                    colSpan && `
                      univer-col-span-${colSpan}
                    `,
                    rowSpan && `
                      univer-row-span-${rowSpan}
                    `,
                    // Custom styles
                    className
                )}
                {...rest}
            >
                {children}
            </button>
        );
    }
);

KeyboardItem.displayName = 'KeyboardItem';
