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

import type { IKeyboardConfirmAndMoveOperationParams, IKeyboardSetModeOperationParams } from '../../../commands/operations/keyboard.operation';
import { Direction, ICommandService } from '@univerjs/core';
import { Button } from '@univerjs/design';
import { useDependency, useObservable } from '@univerjs/ui';
import { useCallback } from 'react';
import {
    KeyboardConfirmAndMoveOperation,
    KeyboardSetModeOperation,
} from '../../../commands/operations/keyboard.operation';
import { IKeyboardService, KeyboardMode } from '../../../services/keyboard.service';

export function ModeSwitcher() {
    const commandService = useDependency(ICommandService);
    const keyboardService = useDependency(IKeyboardService);
    const currentMode = useObservable(keyboardService.keyboardMode$, KeyboardMode.NUMBER);

    const handleTab = useCallback(async () => {
        commandService.executeCommand(KeyboardConfirmAndMoveOperation.id, {
            direction: Direction.RIGHT,
        } satisfies IKeyboardConfirmAndMoveOperationParams);
    }, [commandService]);

    const handleModeChange = useCallback((mode: KeyboardMode) => {
        commandService.executeCommand(KeyboardSetModeOperation.id, {
            mode,
        } satisfies IKeyboardSetModeOperationParams);
    }, [commandService]);

    const handleEnter = useCallback(() => {
        commandService.executeCommand(KeyboardConfirmAndMoveOperation.id, {
            direction: Direction.DOWN,
        } satisfies IKeyboardConfirmAndMoveOperationParams);
    }, [commandService]);

    return (
        <div
            className={`
              univer-flex univer-items-center univer-gap-1 univer-border-b univer-border-gray-200 univer-bg-gray-100
              univer-px-2 univer-py-2
              dark:!univer-border-gray-700 dark:!univer-bg-gray-900
            `}
        >
            <Button
                type="button"
                className="univer-flex-1"
                variant="default"
                onClick={handleTab}
            >
                Tab
            </Button>

            <Button
                type="button"
                className="univer-flex-1"
                variant={currentMode === 'formula' ? 'primary' : 'default'}
                onClick={() => handleModeChange(KeyboardMode.FORMULA)}
            >
                f(x)
            </Button>

            <Button
                type="button"
                className="univer-flex-1"
                variant={currentMode === 'number' ? 'primary' : 'default'}
                onClick={() => handleModeChange(KeyboardMode.NUMBER)}
            >
                123
            </Button>

            <Button
                type="button"
                className="univer-flex-1"
                variant={currentMode === 'text' ? 'primary' : 'default'}
                onClick={() => handleModeChange(KeyboardMode.TEXT)}
            >
                ABC
            </Button>

            <Button
                type="button"
                className="univer-flex-1"
                variant="default"
                onClick={handleEnter}
            >
                ↵
            </Button>
        </div>
    );
}
