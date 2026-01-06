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
import { useDependency } from '@univerjs/ui';
import { useMemo } from 'react';
import { FindBy, FindDirection, FindScope } from '../../../services/find-replace.service';

/**
 * Hook that provides find-replace option arrays for both desktop and mobile.
 * This hook extracts option generation logic to enable code reuse.
 */
export function useFindReplaceOptions() {
    const localeService = useDependency(LocaleService);
    const locale = localeService.getCurrentLocale();

    const findScopeOptions = useMemo(() => {
        return [
            { label: localeService.t('find-replace.dialog.find-scope.current-sheet'), value: FindScope.SUBUNIT },
            { label: localeService.t('find-replace.dialog.find-scope.workbook'), value: FindScope.UNIT },
        ];
    }, [locale, localeService]);

    const findDirectionOptions = useMemo(() => {
        return [
            { label: localeService.t('find-replace.dialog.find-direction.row'), value: FindDirection.ROW },
            { label: localeService.t('find-replace.dialog.find-direction.column'), value: FindDirection.COLUMN },
        ];
    }, [locale, localeService]);

    const findByOptions = useMemo(() => {
        return [
            { label: localeService.t('find-replace.dialog.find-by.value'), value: FindBy.VALUE },
            { label: localeService.t('find-replace.dialog.find-by.formula'), value: FindBy.FORMULA },
        ];
    }, [locale, localeService]);

    return {
        findScopeOptions,
        findDirectionOptions,
        findByOptions,
    };
}
