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

import { clsx } from '@univerjs/design';
import { IEditorService } from '@univerjs/docs-ui';
import { useDependency, useObservable } from '@univerjs/ui';
import { IMobileKeyboardService } from '../../../../services/mobile/mobile-keyboard.service';
import { FormulaKeyboard } from '../formula-keyboard/FormulaKeyboard';
import { NumberKeyboard } from '../number-keyboard/NumberKeyboard';
import { TextKeyboard } from '../text-keyboard/TextKeyboard';
import { KeyboardItem } from './KeyboardItem';
import { MobileFormulaBar } from './MobileFormulaBar';
import { ModeSwitcher } from './ModeSwitcher';
import { OperationToolbar } from './OperationToolbar';

export { KeyboardItem };

export function KeyboardContainer() {
    const mobileKeyboardService = useDependency(IMobileKeyboardService);
    const isVisible = useObservable(mobileKeyboardService.isKeyboardVisible$, false);
    const currentMode = useObservable(mobileKeyboardService.keyboardMode$, 'number');

    const editorService = useDependency(IEditorService);
    if (!isVisible) {
        return null;
    }

    const handleSwipeDown = () => {
        editorService.blur();
        mobileKeyboardService.hideKeyboard();
    };

    return (
        <div
            className={clsx(
                // Use sticky instead of fixed to allow keyboard toolbar to move up
                'univer-sticky univer-bottom-0 univer-left-0 univer-right-0 univer-z-50',
                `
                  univer-bg-white
                  dark:!univer-bg-gray-900
                `,
                `
                  univer-border-t univer-border-gray-200 univer-shadow-lg
                  dark:!univer-border-gray-700
                `,
                'univer-transition-transform univer-duration-200 univer-ease-in-out',
                // Slide up animation
                'univer-translate-y-0'
            )}
        >
            {/* Swipe-down indicator area */}
            <div
                className={`
                  univer-flex univer-h-2 univer-w-full univer-cursor-pointer univer-items-center univer-justify-center
                `}
                onClick={handleSwipeDown}
            >
                <div
                    className={`
                      univer-h-1 univer-w-12 univer-rounded-full univer-bg-gray-300
                      dark:!univer-bg-gray-600
                    `}
                />
            </div>

            {/* Operation Toolbar: Undo, Redo, Copy, Paste, Cut, Clear */}
            <OperationToolbar />

            {/* Mobile FormulaBar */}
            <div className="univer-px-2">
                <MobileFormulaBar />
            </div>

            {/* Mode Switcher: Tab, f(x), 123, ABC, Enter */}
            <ModeSwitcher />

            {/* Mode-specific keyboard */}
            <div className="univer-flex-1 univer-overflow-auto">
                {currentMode === 'formula' && <FormulaKeyboard />}
                {currentMode === 'number' && <NumberKeyboard />}
                {currentMode === 'text' && <TextKeyboard />}
            </div>
        </div>
    );
}
