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

import { LocaleService } from '@univerjs/core';
import { Button, Checkbox, FormLayout, Input, Select } from '@univerjs/design';
import { useDependency } from '@univerjs/ui';
import { IFindReplaceService } from '../../services/find-replace.service';
import { useFindReplaceLogic } from '../dialog/hooks/use-find-replace-logic';
import { useFindReplaceOptions } from '../dialog/hooks/use-find-replace-options';
import { SearchInput } from '../dialog/SearchInput';

/**
 * Mobile find-replace component that renders within a sidebar.
 * Uses shared hooks for business logic to ensure code reuse with desktop implementation.
 */
export function MobileFindReplace() {
    const localeService = useDependency(LocaleService);
    const findReplaceService = useDependency(IFindReplaceService);
    const state = useFindReplaceLogic();
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
        findDisabled,
        replaceDisabled,
        replaceAllDisabled,
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
    } = state;

    const { findScopeOptions, findDirectionOptions, findByOptions } = useFindReplaceOptions();

    // Render find-only mode
    if (!state.state.replaceRevealed) {
        return (
            <div data-u-comp="find-replace-mobile" className="univer-flex univer-flex-col univer-gap-4">
                <SearchInput
                    findCompleted={findCompleted}
                    matchesCount={matchesCount}
                    matchesPosition={matchesPosition}
                    findReplaceService={findReplaceService}
                    localeService={localeService}
                    initialFindString={findString}
                    onChange={onFindStringChange}
                />
                <div className="univer-py-4 univer-text-center">
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
            </div>
        );
    }

    // Render replace mode
    return (
        <div data-u-comp="find-replace-mobile" className="univer-flex univer-flex-col univer-gap-4">
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
            <FormLayout label={localeService.t('find-replace.dialog.find-scope.title')}>
                <Select value={findScope} options={findScopeOptions} onChange={onChangeFindScope} />
            </FormLayout>
            <FormLayout label={localeService.t('find-replace.dialog.find-by.title')}>
                <Select value={findBy} options={findByOptions} onChange={onChangeFindBy} />
            </FormLayout>
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
            <div
                className={`
                  univer-sticky univer-bottom-0 univer-flex univer-flex-col univer-gap-3 univer-bg-white univer-py-4
                  dark:univer-bg-gray-900
                `}
            >
                <Button
                    variant="primary"
                    onClick={onClickFindButton}
                    disabled={findDisabled}
                    className="univer-w-full"
                >
                    {localeService.t('find-replace.dialog.find')}
                </Button>
                <div className="univer-flex univer-gap-3">
                    <Button
                        disabled={replaceDisabled}
                        onClick={onClickReplaceButton}
                        className="univer-flex-1"
                    >
                        {localeService.t('find-replace.dialog.replace')}
                    </Button>
                    <Button
                        disabled={replaceAllDisabled}
                        onClick={onClickReplaceAllButton}
                        className="univer-flex-1"
                    >
                        {localeService.t('find-replace.dialog.replace-all')}
                    </Button>
                </div>
            </div>
        </div>
    );
}

// Register component key for ComponentManager
MobileFindReplace.componentKey = 'univer.find-replace.mobile';
