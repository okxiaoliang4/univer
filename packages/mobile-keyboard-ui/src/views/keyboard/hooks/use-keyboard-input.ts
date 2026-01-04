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

import type { IKeyboardConfirmAndMoveOperationParams, IKeyboardInsertFunctionOperationParams, IKeyboardInsertQuotesOperationParams, IKeyboardInsertTextOperationParams, IKeyboardSetModeOperationParams } from '../../../commands/operations/keyboard.operation';
import type { KeyboardMode } from '../../../services/mobile-keyboard.service';
import { Direction, ICommandService } from '@univerjs/core';
import { useDependency } from '@univerjs/ui';
import { useCallback } from 'react';
import {
    KeyboardConfirmAndMoveOperation,
    KeyboardDeleteBackwardOperation,
    KeyboardInsertFunctionOperation,
    KeyboardInsertQuotesOperation,
    KeyboardInsertTextOperation,
    KeyboardSetModeOperation,
} from '../../../commands/operations/keyboard.operation';
import { IMobileKeyboardService } from '../../../services/mobile-keyboard.service';

/**
 * Hook to handle keyboard input for mobile keyboards
 * Provides methods to insert text, delete, and navigate in the FormulaBar
 */

// eslint-disable-next-line max-lines-per-function
export function useKeyboardInput() {
    const commandService = useDependency(ICommandService);
    const mobileKeyboardService = useDependency(IMobileKeyboardService);
    /**
     * Insert text into the FormulaBar editor at current cursor position
     */
    const insertText = useCallback(
        (text: string) => commandService.executeCommand(KeyboardInsertTextOperation.id, {
            text,
        } satisfies IKeyboardInsertTextOperationParams),
        [commandService]
    );

    /**
     * Delete character before cursor
     */
    const deleteBackward = useCallback(() => {
        commandService.executeCommand(KeyboardDeleteBackwardOperation.id);
    }, [commandService]);

    /**
     * Confirm input and move to next cell
     */
    const confirmAndMove = useCallback(
        (direction: Direction.DOWN | Direction.RIGHT = Direction.DOWN) => {
            commandService.executeCommand(KeyboardConfirmAndMoveOperation.id, {
                direction,
            } satisfies IKeyboardConfirmAndMoveOperationParams);
        },
        [commandService]
    );

    /**
     * Insert function and move cursor inside parentheses
     */
    const insertFunction = useCallback(
        (funcName: string) => {
            commandService.executeCommand(KeyboardInsertFunctionOperation.id, {
                funcName,
            } satisfies IKeyboardInsertFunctionOperationParams);
        },
        [mobileKeyboardService]
    );

    /**
     * Insert double quotes and move cursor between them
     */
    const insertQuotes = useCallback(
        () => {
            commandService.executeCommand(KeyboardInsertQuotesOperation.id, {
                quotes: '""',
            } satisfies IKeyboardInsertQuotesOperationParams);
        },
        [commandService]
    );

    /**
     * Set the active keyboard mode
     */
    const setMode = useCallback(
        (mode: KeyboardMode) => {
            commandService.executeCommand(KeyboardSetModeOperation.id, {
                mode,
            } satisfies IKeyboardSetModeOperationParams);
        },
        [commandService]
    );

    /**
     * Toggle negative sign
     */
    const negativeNumber = useCallback(() => {
        mobileKeyboardService.negativeNumber();
    }, [mobileKeyboardService]);

    return {
        insertText,
        deleteBackward,
        confirmAndMove,
        insertFunction,
        insertQuotes,
        setMode,
        negativeNumber,
    };
}
