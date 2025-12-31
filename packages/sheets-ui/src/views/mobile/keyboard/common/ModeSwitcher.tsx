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

import type { KeyboardMode } from '../../../../services/mobile/mobile-keyboard.service';
import { Direction } from '@univerjs/core';
import { clsx } from '@univerjs/design';
import { useDependency, useObservable } from '@univerjs/ui';
import { useCallback } from 'react';
import { IMobileKeyboardService } from '../../../../services/mobile/mobile-keyboard.service';
import { useKeyboardInput } from '../hooks/use-keyboard-input';

export function ModeSwitcher() {
    const { confirmAndMove } = useKeyboardInput();
    const mobileKeyboardService = useDependency(IMobileKeyboardService);
    const currentMode = useObservable(mobileKeyboardService.keyboardMode$, 'number');

    const handleTab = useCallback(async () => {
        confirmAndMove(Direction.RIGHT);
    }, [confirmAndMove]);

    const handleModeChange = useCallback((mode: KeyboardMode) => {
        mobileKeyboardService.setMode(mode);
    }, [mobileKeyboardService]);

    const handleEnter = useCallback(() => {
        confirmAndMove(Direction.DOWN);
    }, [confirmAndMove]);

    return (
        <div
            className={`
              univer-flex univer-items-center univer-gap-1 univer-border-b univer-border-gray-200 univer-bg-gray-100
              univer-px-2 univer-py-2
              dark:!univer-border-gray-700 dark:!univer-bg-gray-900
            `}
        >
            <button
                type="button"
                className={clsx(
                    `
                      univer-flex-1 univer-rounded-md univer-border-none univer-px-3 univer-py-2 univer-text-sm
                      univer-font-medium
                    `,
                    `
                      univer-bg-white univer-transition-colors
                      dark:!univer-bg-gray-800
                    `,
                    `
                      univer-text-gray-700
                      dark:!univer-text-gray-300
                    `
                )}
                onClick={handleTab}
            >
                Tab
            </button>

            <button
                type="button"
                className={clsx(
                    `
                      univer-flex-1 univer-rounded-md univer-border-none univer-px-3 univer-py-2 univer-text-sm
                      univer-font-medium
                    `,
                    'univer-transition-colors',
                    {
                        'univer-bg-blue-500 univer-text-white dark:!univer-bg-blue-600': currentMode === 'formula',
                        'univer-bg-white univer-text-gray-700 dark:!univer-bg-gray-800 dark:!univer-text-gray-300': currentMode !== 'formula',
                    }
                )}
                onClick={() => handleModeChange('formula')}
            >
                f(x)
            </button>

            <button
                type="button"
                className={clsx(
                    `
                      univer-flex-1 univer-rounded-md univer-border-none univer-px-3 univer-py-2 univer-text-sm
                      univer-font-medium
                    `,
                    'univer-transition-colors',
                    {
                        'univer-bg-blue-500 univer-text-white dark:!univer-bg-blue-600': currentMode === 'number',
                        'univer-bg-white univer-text-gray-700 dark:!univer-bg-gray-800 dark:!univer-text-gray-300': currentMode !== 'number',
                    }
                )}
                onClick={() => handleModeChange('number')}
            >
                123
            </button>

            <button
                type="button"
                className={clsx(
                    `
                      univer-flex-1 univer-rounded-md univer-border-none univer-px-3 univer-py-2 univer-text-sm
                      univer-font-medium
                    `,
                    'univer-transition-colors',
                    {
                        'univer-bg-blue-500 univer-text-white dark:!univer-bg-blue-600': currentMode === 'text',
                        'univer-bg-white univer-text-gray-700 dark:!univer-bg-gray-800 dark:!univer-text-gray-300': currentMode !== 'text',
                    }
                )}
                onClick={() => handleModeChange('text')}
            >
                ABC
            </button>

            <button
                type="button"
                className={clsx(
                    `
                      univer-flex-1 univer-rounded-md univer-border-none univer-px-3 univer-py-2 univer-text-sm
                      univer-font-medium
                    `,
                    `
                      univer-bg-white univer-transition-colors
                      dark:!univer-bg-gray-800
                    `,
                    `
                      univer-text-gray-700
                      dark:!univer-text-gray-300
                    `
                )}
                onClick={handleEnter}
            >
                ↵
            </button>
        </div>
    );
}
