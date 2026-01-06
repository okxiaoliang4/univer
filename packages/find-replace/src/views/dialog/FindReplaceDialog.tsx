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

import type { IDisposable } from '@univerjs/core';
import type { ISubFormRef } from './hooks/use-find-replace-logic';
import { IContextService, LocaleService } from '@univerjs/core';
import { Button, Checkbox, FormDualColumnLayout, FormLayout, Input, Select } from '@univerjs/design';
import { ILayoutService, useDependency, useObservable } from '@univerjs/ui';
import { forwardRef, useCallback, useEffect, useRef } from 'react';
import { fromEvent } from 'rxjs';
import { FIND_REPLACE_DIALOG_FOCUS, FIND_REPLACE_INPUT_FOCUS } from '../../services/context-keys';
import { IFindReplaceService } from '../../services/find-replace.service';
import { useFindReplaceLogic } from './hooks/use-find-replace-logic';
import { useFindReplaceOptions } from './hooks/use-find-replace-options';
import { SearchInput } from './SearchInput';

export const FindDialog = forwardRef<ISubFormRef>(function FindDialogImpl(_props, ref) {
    const localeService = useDependency(LocaleService);
    const findReplaceService = useDependency(IFindReplaceService);
    const {
        findCompleted,
        findString,
        matchesCount,
        matchesPosition,
        revealReplace,
        onFindStringChange,
    } = useFindReplaceLogic(ref);

    return (
        <>
            <SearchInput
                findCompleted={findCompleted}
                matchesCount={matchesCount}
                matchesPosition={matchesPosition}
                findReplaceService={findReplaceService}
                localeService={localeService}
                initialFindString={findString}
                onChange={onFindStringChange}
            />
            <div className="univer-mt-4 univer-text-center">
                <a
                    className={`
                      hover:univer-text-primary-500/80
                      univer-cursor-pointer univer-text-sm univer-text-primary-500 univer-transition-colors
                    `}
                    onClick={revealReplace}
                >
                    {localeService.t('find-replace.dialog.advanced-finding')}
                </a>
            </div>
        </>
    );
});

export const ReplaceDialog = forwardRef<ISubFormRef>(function ReplaceDialogImpl(_props, ref) {
    const findReplaceService = useDependency(IFindReplaceService);
    const localeService = useDependency(LocaleService);

    const {
        matchesCount,
        matchesPosition,
        inputtingFindString,
        replaceString,
        caseSensitive,
        matchesTheWholeCell,
        findDirection,
        findScope,
        findBy,
        findCompleted,
        findDisabled,
        replaceDisabled,
        replaceAllDisabled,
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
    } = useFindReplaceLogic(ref);

    const { findScopeOptions, findDirectionOptions, findByOptions } = useFindReplaceOptions();

    return (
        <div>
            <FormLayout label={localeService.t('find-replace.dialog.find')}>
                <SearchInput
                    findCompleted={findCompleted}
                    className="univer-find-input"
                    matchesCount={matchesCount}
                    matchesPosition={matchesPosition}
                    findReplaceService={findReplaceService}
                    localeService={localeService}
                    initialFindString={inputtingFindString}
                    onChange={onFindStringChangeReplace}
                />
            </FormLayout>
            <FormLayout label={localeService.t('find-replace.dialog.replace')}>
                <Input
                    placeholder={localeService.t('find-replace.dialog.replace-placeholder')}
                    value={replaceString}
                    onChange={(value) => onReplaceStringChange(value)}
                />
            </FormLayout>
            <FormLayout label={localeService.t('find-replace.dialog.find-direction.title')}>
                <Select value={findDirection} options={findDirectionOptions} onChange={onChangeFindDirection} />
            </FormLayout>
            <FormDualColumnLayout>
                <>
                    <FormLayout label={localeService.t('find-replace.dialog.find-scope.title')}>
                        <Select value={findScope} options={findScopeOptions} onChange={onChangeFindScope} />
                    </FormLayout>
                    <FormLayout label={localeService.t('find-replace.dialog.find-by.title')}>
                        <Select value={findBy} options={findByOptions} onChange={onChangeFindBy} />
                    </FormLayout>
                </>
            </FormDualColumnLayout>
            <FormDualColumnLayout>
                <>
                    <FormLayout>
                        <Checkbox
                            checked={caseSensitive}
                            onChange={(checked) => onChangeCaseSensitive(checked as boolean)}
                        >
                            {localeService.t('find-replace.dialog.case-sensitive')}
                        </Checkbox>
                    </FormLayout>
                    <FormLayout>
                        <Checkbox
                            checked={matchesTheWholeCell}
                            onChange={(checked) => onChangeMatchesTheWholeCell(checked as boolean)}
                        >
                            {localeService.t('find-replace.dialog.match-the-whole-cell')}
                        </Checkbox>
                    </FormLayout>
                </>
            </FormDualColumnLayout>
            <div className="univer-mt-6 univer-flex univer-justify-between">
                <Button variant="primary" onClick={onClickFindButton} disabled={findDisabled}>{localeService.t('find-replace.dialog.find')}</Button>
                <span className="univer-inline-flex univer-gap-2">
                    <Button disabled={replaceDisabled} onClick={onClickReplaceButton}>{localeService.t('find-replace.dialog.replace')}</Button>
                    <Button disabled={replaceAllDisabled} onClick={onClickReplaceAllButton}>{localeService.t('find-replace.dialog.replace-all')}</Button>
                </span>
            </div>
        </div>
    );
});

export function FindReplaceDialog() {
    const findReplaceService = useDependency(IFindReplaceService);
    const layoutService = useDependency(ILayoutService);
    const contextService = useDependency(IContextService);

    const state = useObservable(findReplaceService.state$, undefined, true);

    const dialogContainerRef = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let disposable: IDisposable | undefined;
        if (dialogContainerRef.current) {
            disposable = layoutService.registerContainerElement(dialogContainerRef.current);
        }

        return () => disposable?.dispose();
    }, [layoutService]);

    const focusRef = useRef<ISubFormRef>(null);
    const setDialogContainerFocus = useCallback(
        (focused: boolean) => contextService.setContextValue(FIND_REPLACE_DIALOG_FOCUS, focused),
        [contextService]
    );
    const setDialogInputFocus = useCallback(
        (focused: boolean) => contextService.setContextValue(FIND_REPLACE_INPUT_FOCUS, focused),
        [contextService]
    );

    useEffect(() => {
        const focusSubscription = fromEvent(document, 'focusin').subscribe((event) => {
            if (event.target && dialogContainerRef.current?.contains(event.target as Node)) {
                setDialogContainerFocus(true);
            } else {
                setDialogContainerFocus(false);
            }

            if (!focusRef.current || !focusRef.current.selectHasFocus()) {
                setDialogInputFocus(false);
            } else {
                setDialogInputFocus(true);
            }
        });

        // Focus the input element the first time we open the find replace dialog.
        focusRef.current?.focus();

        setDialogContainerFocus(true);
        setDialogInputFocus(true);

        return () => {
            focusSubscription.unsubscribe();
            setDialogContainerFocus(false);
        };
    }, [setDialogContainerFocus, setDialogInputFocus]);

    return (
        <div ref={dialogContainerRef} data-u-comp="find-replace-dialog">
            {!state.replaceRevealed ? <FindDialog ref={focusRef} /> : <ReplaceDialog ref={focusRef} />}
        </div>
    );
}
