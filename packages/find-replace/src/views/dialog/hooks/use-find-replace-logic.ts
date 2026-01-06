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

import type { ForwardedRef } from 'react';
import type { FindBy, FindDirection, FindScope } from '../../../services/find-replace.service';
import { ICommandService, LocaleService } from '@univerjs/core';
import { MessageType } from '@univerjs/design';
import { IMessageService, useDebounceFn, useDependency, useObservable } from '@univerjs/ui';
import { useCallback, useEffect, useImperativeHandle } from 'react';
import { ReplaceAllMatchesCommand, ReplaceCurrentMatchCommand } from '../../../commands/commands/replace.command';
import { OpenReplaceDialogOperation } from '../../../commands/operations/find-replace.operation';
import { IFindReplaceService } from '../../../services/find-replace.service';

export interface ISubFormRef {
    focus(): void;
    selectHasFocus(): boolean;
}

/**
 * Hook that extracts find-replace business logic for reuse between desktop and mobile.
 * This hook provides state management, event handlers, and focus management.
 */
// eslint-disable-next-line max-lines-per-function
export function useFindReplaceLogic(ref?: ForwardedRef<ISubFormRef>) {
    const localeService = useDependency(LocaleService);
    const findReplaceService = useDependency(IFindReplaceService);
    const commandService = useDependency(ICommandService);
    const messageService = useDependency(IMessageService);

    const state = useObservable(findReplaceService.state$, undefined, true);
    const currentMatch = useObservable(findReplaceService.currentMatch$, undefined, true);
    const replaceables = useObservable(findReplaceService.replaceables$, undefined, true);

    const {
        findCompleted,
        findString,
        inputtingFindString,
        replaceString,
        caseSensitive,
        matchesTheWholeCell,
        findDirection,
        findScope,
        findBy,
        matchesCount,
        matchesPosition,
    } = state;

    const findDisabled = inputtingFindString.length === 0;
    const replaceDisabled = matchesCount === 0 || !currentMatch?.replaceable;
    const replaceAllDisabled = replaceables.length === 0;

    // Focus management
    const focus = useCallback(() => {
        const input = document.querySelector('.univer-find-input input') as HTMLInputElement | null;
        input?.focus();
    }, []);

    const selectHasFocus = useCallback(() => {
        const allInputs = document.querySelectorAll('[data-u-comp=find-replace-dialog] [data-u-comp=search-input], [data-u-comp=find-replace-mobile] [data-u-comp=search-input]');
        return Array.from(allInputs).some((input) => input === document.activeElement);
    }, []);

    // Expose focus methods via ref if provided
    // Note: useImperativeHandle must be called unconditionally, so we always call it
    // but only expose methods if ref is provided
    useImperativeHandle(ref, () => ({ focus, selectHasFocus }), [focus, selectHasFocus]);

    useEffect(() => {
        const subscription = findReplaceService.focusSignal$.subscribe(() => focus());
        return () => subscription.unsubscribe();
    }, [findReplaceService, focus]);

    // Find dialog handlers
    const revealReplace = useCallback(() => {
        commandService.executeCommand(OpenReplaceDialogOperation.id);
    }, [commandService]);

    const onFindStringChange = useDebounceFn((findString: string) => {
        return findReplaceService.changeFindString(findString);
    }, 500);

    // Replace dialog handlers
    const onFindStringChangeReplace = useCallback(
        (newValue: string) => findReplaceService.changeInputtingFindString(newValue),
        [findReplaceService]
    );

    const onReplaceStringChange = useCallback(
        (replaceString: string) => findReplaceService.changeReplaceString(replaceString),
        [findReplaceService]
    );

    const onClickFindButton = useCallback(() => {
        if (findString === inputtingFindString) {
            findReplaceService.moveToNextMatch();
        } else {
            findReplaceService.changeFindString(inputtingFindString);
            findReplaceService.find();
        }
    }, [findString, inputtingFindString, findReplaceService]);

    const onClickReplaceButton = useCallback(() => {
        commandService.executeCommand(ReplaceCurrentMatchCommand.id);
    }, [commandService]);

    const onClickReplaceAllButton = useCallback(async () => {
        await commandService.executeCommand(ReplaceAllMatchesCommand.id);
        focus();
    }, [commandService, focus]);

    const onChangeFindDirection = useCallback((findDirection: string) => {
        findReplaceService.changeFindDirection(findDirection as FindDirection);
    }, [findReplaceService]);

    const onChangeFindScope = useCallback((findScope: string) => {
        findReplaceService.changeFindScope(findScope as FindScope);
    }, [findReplaceService]);

    const onChangeFindBy = useCallback((findBy: string) => {
        findReplaceService.changeFindBy(findBy as FindBy);
    }, [findReplaceService]);

    const onChangeCaseSensitive = useCallback((checked: boolean) => {
        findReplaceService.changeCaseSensitive(checked);
    }, [findReplaceService]);

    const onChangeMatchesTheWholeCell = useCallback((checked: boolean) => {
        findReplaceService.changeMatchesTheWholeCell(checked);
    }, [findReplaceService]);

    // Show no match message
    useEffect(() => {
        const shouldDisplayNoMatchInfo = findCompleted && matchesCount === 0;

        if (shouldDisplayNoMatchInfo) {
            messageService.show({
                content: localeService.t('find-replace.dialog.no-match'),
                type: MessageType.Warning,
                duration: 5000,
            });
        }
    }, [findCompleted, matchesCount, messageService, localeService]);

    return {
        // State
        state,
        currentMatch,
        replaceables,
        findCompleted,
        findString,
        inputtingFindString,
        replaceString,
        caseSensitive,
        matchesTheWholeCell,
        findDirection,
        findScope,
        findBy,
        matchesCount,
        matchesPosition,
        // Computed
        findDisabled,
        replaceDisabled,
        replaceAllDisabled,
        // Handlers
        revealReplace,
        onFindStringChange,
        onFindStringChangeReplace,
        onReplaceStringChange,
        onClickFindButton,
        onClickReplaceButton,
        onClickReplaceAllButton,
        onChangeFindDirection,
        onChangeFindScope,
        onChangeFindBy,
        onChangeCaseSensitive,
        onChangeMatchesTheWholeCell,
        // Focus management
        focus,
        selectHasFocus,
    };
}
