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

import { Direction } from '@univerjs/core';
import { KeyboardItem } from '../common/KeyboardItem';
import { useKeyboardInput } from '../hooks/use-keyboard-input';

export function NumberKeyboard() {
    const { insertText, deleteBackward, confirmAndMove, negativeNumber } = useKeyboardInput();

    const handleKeyPress = (value: string) => {
        // Special keys
        switch (value) {
            case 'del':
                deleteBackward();
                break;
            case 'enter':
                confirmAndMove(Direction.DOWN);
                break;
            case 'tab':
                confirmAndMove(Direction.RIGHT);
                break;
            case '+/-': {
                // Toggle negative sign logic could be complex, let's just insert it for now
                negativeNumber();
                break;
            }
            default:
                insertText(value);
        }
    };

    return (
        <div
            className={`
              univer-bg-gray-50 univer-p-2
              dark:!univer-bg-gray-800
            `}
        >

            <div className="univer-grid univer-grid-cols-5 univer-gap-1">
                <KeyboardItem
                    className="univer-p-2 univer-text-lg"
                    onClick={() => handleKeyPress('+/-')}
                >
                    +/-
                </KeyboardItem>
                {['7', '8', '9'].map((key) => (
                    <KeyboardItem
                        key={key}
                        className="univer-p-2 univer-text-xl univer-font-medium"
                        onClick={() => handleKeyPress(key)}
                    >
                        {key}
                    </KeyboardItem>
                ))}
                <KeyboardItem
                    variant="danger"
                    className="univer-p-2 univer-text-base"
                    onClick={() => handleKeyPress('del')}
                >
                    ⌫
                </KeyboardItem>

                <KeyboardItem
                    className="univer-p-2 univer-text-lg"
                    onClick={() => handleKeyPress('%')}
                >
                    %
                </KeyboardItem>
                {['4', '5', '6'].map((key) => (
                    <KeyboardItem
                        key={key}
                        className="univer-p-2 univer-text-xl univer-font-medium"
                        onClick={() => handleKeyPress(key)}
                    >
                        {key}
                    </KeyboardItem>
                ))}
                <KeyboardItem
                    className="univer-p-2 univer-text-sm"
                    onClick={() => handleKeyPress('tab')}
                >
                    Tab
                </KeyboardItem>
                <KeyboardItem
                    className="univer-p-2 univer-text-lg"
                    onClick={() => handleKeyPress('$')}
                >
                    $
                </KeyboardItem>
                {['1', '2', '3'].map((key) => (
                    <KeyboardItem
                        key={key}
                        className="univer-p-2 univer-text-xl univer-font-medium"
                        onClick={() => handleKeyPress(key)}
                    >
                        {key}
                    </KeyboardItem>
                ))}
                <KeyboardItem
                    variant="primary"
                    className="univer-row-span-2 univer-h-auto univer-p-2 univer-text-base"
                    onClick={() => handleKeyPress('enter')}
                >
                    ↵
                </KeyboardItem>
                <KeyboardItem
                    className="univer-p-2 univer-text-lg"
                    onClick={() => handleKeyPress('¥')}
                >
                    ¥
                </KeyboardItem>
                <KeyboardItem
                    className="univer-p-2 univer-text-base"
                    onClick={() => handleKeyPress('00')}
                >
                    00
                </KeyboardItem>
                {['0', '.'].map((key) => (
                    <KeyboardItem
                        key={key}
                        className="univer-p-2 univer-text-xl univer-font-medium"
                        onClick={() => handleKeyPress(key)}
                    >
                        {key}
                    </KeyboardItem>
                ))}
            </div>
        </div>
    );
}
