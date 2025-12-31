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

import type { IButtonProps } from '@univerjs/design';
import type { ReactNode } from 'react';
import { Button, clsx } from '@univerjs/design';
import { forwardRef } from 'react';

export interface IKeyboardItemProps extends Omit<IButtonProps, 'className'> {
    /** Additional className for custom styling */
    className?: string;
    /** Button content */
    children: ReactNode;
}

export const KeyboardItem = forwardRef<HTMLButtonElement, IKeyboardItemProps>(
    ({ className, type = 'button', variant, children, ...rest }, ref) => {
        return (
            <Button
                ref={ref}
                type={type}
                variant={variant}
                className={clsx('univer-border-none', className)}
                {...rest}
            >
                {children}
            </Button>
        );
    }
);

KeyboardItem.displayName = 'KeyboardItem';
