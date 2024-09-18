/**
 * Copyright 2023-present DreamNum Inc.
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

import { NumberValueObject } from '../../../engine/value-object/primitive-object';
import { BaseFunction } from '../../base-function';
import type { ArrayValueObject } from '../../../engine/value-object/array-value-object';
import type { BaseValueObject } from '../../../engine/value-object/base-value-object';

export class Int extends BaseFunction {
    override minParams = 1;

    override maxParams = 1;

    override calculate(number: BaseValueObject): BaseValueObject {
        if (number.isArray()) {
            return (number as ArrayValueObject).mapValue((numberObject) => this._handleSingleObject(numberObject));
        }

        return this._handleSingleObject(number);
    }

    private _handleSingleObject(number: BaseValueObject): BaseValueObject {
        let _number = number;

        if (_number.isString()) {
            _number = _number.convertToNumberObjectValue();
        }

        if (_number.isError()) {
            return _number;
        }

        const numberValue = Math.floor(+_number.getValue());

        return NumberValueObject.create(numberValue);
    }
}