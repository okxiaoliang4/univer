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

import { IUniverInstanceService, UniverInstanceType } from '@univerjs/core';
import { clsx } from '@univerjs/design';
import { useDependency, useObservable } from '@univerjs/ui';
import { useCallback } from 'react';
import { IMobileKeyboardService } from '../../../services/mobile/mobile-keyboard.service';

export function KeyboardFab() {
    const mobileKeyboardService = useDependency(IMobileKeyboardService);
    const univerInstanceService = useDependency(IUniverInstanceService);

    const isKeyboardVisible = useObservable(mobileKeyboardService.isKeyboardVisible$, false);

    const handleFabClick = useCallback(() => {
        const workbook = univerInstanceService.getCurrentUnitForType(UniverInstanceType.UNIVER_SHEET);
        if (!workbook) {
            return;
        }

        // Trigger cell edit using the existing operation
        mobileKeyboardService.showKeyboard();
    }, [mobileKeyboardService, univerInstanceService]);

    // FAB is only visible when keyboard is NOT visible
    if (isKeyboardVisible) {
        return null;
    }

    return (
        <div
            className={clsx(
                'univer-fixed univer-bottom-12 univer-right-4 univer-z-[9999]',
                'univer-flex univer-select-none'
            )}
        >
            <button
                type="button"
                className={clsx(
                    'univer-flex univer-items-center univer-justify-center',
                    'univer-h-14 univer-w-14 univer-rounded-full',
                    `
                      univer-bg-blue-500
                      hover:univer-bg-blue-600
                    `,
                    'univer-text-2xl univer-text-white',
                    'univer-shadow-lg',
                    'univer-transition-all univer-duration-200',
                    `
                      hover:univer-scale-105
                      active:univer-scale-95
                    `,
                    'focus:univer-outline-none focus:univer-ring-2 focus:univer-ring-blue-300'
                )}
                onClick={handleFabClick}
                title="Open keyboard"
            >
                ⌨️
            </button>
        </div>
    );
}
