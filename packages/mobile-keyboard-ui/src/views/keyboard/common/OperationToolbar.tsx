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

import { ICommandService, IUndoRedoService, LocaleService, RedoCommandId, UndoCommandId } from '@univerjs/core';
import { clsx } from '@univerjs/design';
import { ClearSelectionContentCommand } from '@univerjs/sheets';
import { SheetCopyCommand, SheetCutCommand, SheetPasteCommand } from '@univerjs/sheets-ui';
import { useDependency } from '@univerjs/ui';
import { useCallback, useEffect, useState } from 'react';

export function OperationToolbar() {
    const commandService = useDependency(ICommandService);
    const undoRedoService = useDependency(IUndoRedoService);
    const localeService = useDependency(LocaleService);

    const handleUndo = useCallback(() => {
        commandService.executeCommand(UndoCommandId);
    }, [commandService]);

    const handleRedo = useCallback(() => {
        commandService.executeCommand(RedoCommandId);
    }, [commandService]);

    const handleCopy = useCallback(() => {
        commandService.executeCommand(SheetCopyCommand.id);
    }, [commandService]);

    const handlePaste = useCallback(() => {
        commandService.executeCommand(SheetPasteCommand.id);
    }, [commandService]);

    const handleCut = useCallback(() => {
        commandService.executeCommand(SheetCutCommand.id);
    }, [commandService]);

    const handleClear = useCallback(() => {
        commandService.executeCommand(ClearSelectionContentCommand.id);
    }, [commandService]);

    // Use observables to track undo/redo availability
    const [canUndo, setCanUndo] = useState(false);
    const [canRedo, setCanRedo] = useState(false);

    useEffect(() => {
        const subscriptions = [
            undoRedoService.undoRedoStatus$.subscribe((status) => {
                setCanUndo(status.undos > 0);
                setCanRedo(status.redos > 0);
            }),
        ];
        return () => {
            subscriptions.forEach((s) => s.unsubscribe());
        };
    }, [undoRedoService]);

    const t = (key: string) => localeService.t(key);

    return (
        <div
            className={`
              univer-flex univer-items-center univer-justify-between univer-border-b univer-border-gray-200
              univer-bg-gray-50 univer-px-1 univer-py-2
              dark:!univer-border-gray-700 dark:!univer-bg-gray-800
            `}
        >
            <div className="univer-flex univer-flex-1 univer-gap-2">
                <button
                    type="button"
                    className={clsx(
                        `
                          univer-rounded-md univer-border-none univer-px-2 univer-py-1 univer-text-xs
                          univer-transition-colors
                        `,
                        `
                          univer-bg-white
                          dark:!univer-bg-gray-700
                        `,
                        `
                          hover:univer-bg-gray-100
                          dark:hover:!univer-bg-gray-600
                        `,
                        {
                            'univer-cursor-not-allowed univer-text-gray-400 dark:!univer-text-gray-500': !canUndo,
                            'univer-text-gray-700 dark:!univer-text-gray-300': canUndo,
                        }
                    )}
                    onClick={handleUndo}
                    disabled={!canUndo}
                >
                    {t('toolbar.undo')}
                </button>
                <button
                    type="button"
                    className={clsx(
                        `
                          univer-rounded-md univer-border-none univer-px-2 univer-py-1 univer-text-xs
                          univer-transition-colors
                        `,
                        `
                          univer-bg-white
                          dark:!univer-bg-gray-700
                        `,
                        `
                          hover:univer-bg-gray-100
                          dark:hover:!univer-bg-gray-600
                        `,
                        {
                            'univer-cursor-not-allowed univer-text-gray-400 dark:!univer-text-gray-500': !canRedo,
                            'univer-text-gray-700 dark:!univer-text-gray-300': canRedo,
                        }
                    )}
                    onClick={handleRedo}
                    disabled={!canRedo}
                >
                    {t('toolbar.redo')}
                </button>
                <button
                    type="button"
                    className={clsx(
                        `
                          univer-rounded-md univer-border-none univer-px-2 univer-py-1 univer-text-xs univer-font-medium
                          univer-transition-colors
                        `,
                        `
                          univer-bg-white
                          dark:!univer-bg-gray-700
                        `,
                        `
                          hover:univer-bg-gray-100
                          dark:hover:!univer-bg-gray-600
                        `,
                        `
                          univer-text-gray-700
                          dark:!univer-text-gray-300
                        `
                    )}
                    onClick={handleCopy}
                >
                    {t('rightClick.copy')}
                </button>
                <button
                    type="button"
                    className={clsx(
                        `
                          univer-rounded-md univer-border-none univer-px-2 univer-py-1 univer-text-xs univer-font-medium
                          univer-transition-colors
                        `,
                        `
                          univer-bg-white
                          dark:!univer-bg-gray-700
                        `,
                        `
                          hover:univer-bg-gray-100
                          dark:hover:!univer-bg-gray-600
                        `,
                        `
                          univer-text-gray-700
                          dark:!univer-text-gray-300
                        `
                    )}
                    onClick={handlePaste}
                >
                    {t('rightClick.paste')}
                </button>
                <button
                    type="button"
                    className={clsx(
                        `
                          univer-rounded-md univer-border-none univer-px-2 univer-py-1 univer-text-xs univer-font-medium
                          univer-transition-colors
                        `,
                        `
                          univer-bg-white
                          dark:!univer-bg-gray-700
                        `,
                        `
                          hover:univer-bg-gray-100
                          dark:hover:!univer-bg-gray-600
                        `,
                        `
                          univer-text-gray-700
                          dark:!univer-text-gray-300
                        `
                    )}
                    onClick={handleCut}
                >
                    {t('rightClick.cut')}
                </button>
                <button
                    type="button"
                    className={clsx(
                        `
                          univer-rounded-md univer-border-none univer-px-2 univer-py-1 univer-text-xs univer-font-medium
                          univer-transition-colors
                        `,
                        `
                          univer-bg-white
                          dark:!univer-bg-gray-700
                        `,
                        `
                          hover:univer-bg-gray-100
                          dark:hover:!univer-bg-gray-600
                        `,
                        `
                          univer-text-gray-700
                          dark:!univer-text-gray-300
                        `
                    )}
                    onClick={handleClear}
                >
                    {t('rightClick.delete')}
                </button>
            </div>
        </div>
    );
}
