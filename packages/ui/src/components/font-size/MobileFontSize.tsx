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

import type { IFontSizeProps } from './interface';
import { useMemo, useState } from 'react';

export const MobileFontSize = (props: IFontSizeProps) => {
    const { value } = props;
    const [realValue] = useState<number>(Number(value ?? 0));

    const _value = useMemo(() => Number(value ?? realValue), [value]);

    return (
        <div className="univer-flex univer-items-center univer-gap-2">
            <span className="univer-text-primary-600">{_value}</span>
        </div>
    );
};
