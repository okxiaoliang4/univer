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
import { useState } from 'react';
import { KeyboardItem } from '../common/KeyboardItem';
import { useKeyboardInput } from '../hooks/use-keyboard-input';
import { FunctionBrowser } from './FunctionBrowser';

type SubMode = 'formula' | 'english' | 'function-browser';

export function FormulaKeyboard() {
    const { insertText, deleteBackward, confirmAndMove, insertFunction, insertQuotes } = useKeyboardInput();
    const [subMode, setSubMode] = useState<SubMode>('formula');
    const [isShiftActive, setIsShiftActive] = useState(false);

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
            case 'SUM()':
                insertFunction('SUM');
                break;
            case 'space':
                insertText(' ');
                break;
            case 'quotes':
                insertQuotes();
                break;
            case 'f(x)':
                setSubMode('function-browser');
                break;
            case 'back':
                // setSubMode('formula');
                break;
            case 'shift':
                setIsShiftActive(!isShiftActive);
                break;
            default: {
                let textToInsert = value;
                if (subMode === 'english' && isShiftActive) {
                    textToInsert = textToInsert.toUpperCase();
                }
                insertText(textToInsert);
                if (subMode === 'english' && isShiftActive) {
                    setIsShiftActive(false);
                }
            }
        }
    };

    // if (subMode === 'english') {
    //     return (
    //         <div
    //             className={`
    //               univer-bg-gray-50 univer-p-2
    //               dark:!univer-bg-gray-800
    //             `}
    //         >
    //             {/* English Keyboard Layout */}
    //             <div className="univer-mb-1 univer-grid univer-grid-cols-10 univer-gap-1">
    //                 {['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'].map((key) => (
    //                     <KeyboardItem key={key} className="univer-p-2 univer-text-base" onClick={() => handleKeyPress(key)}>
    //                         {isShiftActive ? key.toUpperCase() : key}
    //                     </KeyboardItem>
    //                 ))}
    //             </div>
    //             <div className="univer-mb-1 univer-flex univer-justify-center univer-gap-1 univer-px-2">
    //                 {['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'].map((key) => (
    //                     <KeyboardItem key={key} className="univer-flex-1 univer-p-2 univer-text-base" onClick={() => handleKeyPress(key)}>
    //                         {isShiftActive ? key.toUpperCase() : key}
    //                     </KeyboardItem>
    //                 ))}
    //             </div>
    //             <div className="univer-mb-1 univer-grid univer-grid-cols-10 univer-gap-1">
    //                 <KeyboardItem className="univer-p-2 univer-text-base" onClick={() => handleKeyPress('shift')}>
    //                     {isShiftActive ? '⬆️' : '⇧'}
    //                 </KeyboardItem>
    //                 {['z', 'x', 'c', 'v', 'b', 'n', 'm'].map((key) => (
    //                     <KeyboardItem key={key} className="univer-p-2 univer-text-base" onClick={() => handleKeyPress(key)}>
    //                         {isShiftActive ? key.toUpperCase() : key}
    //                     </KeyboardItem>
    //                 ))}
    //                 <KeyboardItem variant="danger" className="univer-col-span-2 univer-p-2 univer-text-base" onClick={() => handleKeyPress('del')}>
    //                     ⌫
    //                 </KeyboardItem>
    //             </div>
    //             <div className="univer-grid univer-grid-cols-10 univer-gap-1">
    //                 {['$', ':', ',', '!'].map((key) => (
    //                     <KeyboardItem key={key} className="univer-p-2 univer-text-base" onClick={() => handleKeyPress(key)}>
    //                         {key}
    //                     </KeyboardItem>
    //                 ))}
    //                 <KeyboardItem className="univer-col-span-3 univer-p-2 univer-text-sm" onClick={() => handleKeyPress('space')}>
    //                     space
    //                 </KeyboardItem>
    //                 <KeyboardItem
    //                     type="submit"
    //                     variant="primary"
    //                     className="univer-row-span-2 univer-p-2 univer-text-base"
    //                     onClick={() => handleKeyPress('enter')}
    //                 >
    //                     ↵
    //                 </KeyboardItem>
    //                 <KeyboardItem variant="primary" className="univer-col-span-2 univer-p-2 univer-text-sm" onClick={() => handleKeyPress('back')}>
    //                     back
    //                 </KeyboardItem>
    //             </div>
    //         </div>
    //     );
    // }

    return (
        <>
            <FunctionBrowser
                open={subMode === 'function-browser'}
                onOpenChange={(open) => {
                    if (!open) {
                        setSubMode('formula');
                    }
                }}
                onSelect={(funcName) => {
                    insertFunction(funcName);
                    setSubMode('formula');
                }}
                onClose={() => setSubMode('formula')}
            />
            <div
                className={`
                  univer-bg-gray-50 univer-p-2
                  dark:!univer-bg-gray-800
                `}
            >
                {/* Row 1: Numbers 1-0 */}
                <div className="univer-mb-1 univer-grid univer-grid-cols-10 univer-gap-1">
                    {['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'].map((key) => (
                        <KeyboardItem key={key} className="univer-p-2 univer-text-base" onClick={() => handleKeyPress(key)}>
                            {key}
                        </KeyboardItem>
                    ))}
                </div>
                <div className="univer-grid univer-grid-cols-5 univer-gap-1">
                    <div className="univer-col-span-3 univer-grid univer-grid-cols-4 univer-gap-1">
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress('+')}>+</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress('-')}>−</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress('*')}>×</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress('/')}>÷</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress('+')}>+</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress('(')}>(</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress(')')}>)</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress(',')}>,</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress('<')}>&lt;</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress('>')}>&gt;</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress(':')}>:</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress('.')}>.</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress('$')}>$</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress('%')}>%</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress('&')}>&amp;</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress('^')}>^</KeyboardItem>
                    </div>
                    <div className="univer-col-span-2 univer-grid univer-grid-cols-2 univer-gap-1">
                        <KeyboardItem className="univer-p-2 univer-text-sm" onClick={() => handleKeyPress('f(x)')}>f(x)</KeyboardItem>
                        <KeyboardItem variant="danger" className="univer-p-2 univer-text-base" onClick={() => handleKeyPress('del')}>⌫</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-lg" onClick={() => handleKeyPress('SUM()')}>Σ</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-sm" onClick={() => handleKeyPress('tab')}>Tab</KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-sm" onClick={() => handleKeyPress('quotes')}>&quot;&quot;</KeyboardItem>
                        <KeyboardItem
                            type="submit"
                            variant="primary"
                            className="univer-row-span-2 univer-h-auto univer-p-2 univer-text-base"
                            onClick={() => handleKeyPress('enter')}
                        >
                            ↵
                        </KeyboardItem>
                        <KeyboardItem className="univer-p-2 univer-text-xs" onClick={() => handleKeyPress('space')}>space</KeyboardItem>
                    </div>
                </div>
            </div>
        </>
    );
}
