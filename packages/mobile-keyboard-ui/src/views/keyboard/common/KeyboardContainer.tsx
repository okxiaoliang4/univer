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

import { useDependency, useObservable } from '@univerjs/ui';
import { IMobileKeyboardService, KeyboardMode } from '../../../services/mobile-keyboard.service';
import { FormulaKeyboard } from '../formula-keyboard/FormulaKeyboard';
import { NumberKeyboard } from '../number-keyboard/NumberKeyboard';
import { TextKeyboard } from '../text-keyboard/TextKeyboard';
import { KeyboardItem } from './KeyboardItem';
import { ModeSwitcher } from './ModeSwitcher';

export { KeyboardItem };

export function KeyboardContainer() {
    const mobileKeyboardService = useDependency(IMobileKeyboardService);
    const isVisible = useObservable(mobileKeyboardService.isKeyboardVisible$, false);
    const currentMode = useObservable(mobileKeyboardService.keyboardMode$, KeyboardMode.NUMBER);

    if (!isVisible) {
        return null;
    }

    const stopEventHandler = (e: React.MouseEvent<HTMLDivElement> | React.PointerEvent<HTMLDivElement>) => {
        e.preventDefault();
        e.stopPropagation();
    };

    return (
        <div
            onClick={stopEventHandler}
            onPointerDown={stopEventHandler}
            className={`
              univer-translate-y-0 univer-border-t univer-border-gray-200 univer-bg-white univer-shadow-lg
              univer-transition-transform univer-duration-200 univer-ease-in-out
              dark:!univer-border-gray-700 dark:!univer-bg-gray-900
            `}
        >
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
