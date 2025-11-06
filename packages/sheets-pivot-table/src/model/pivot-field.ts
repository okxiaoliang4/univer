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

import type { IPivotField, IPivotFilterCriteria } from '../types/type';
import { Disposable } from '@univerjs/core';
import { AggregationType, PivotFieldAreaType, PivotSortOrder } from '../types/enum';

export class PivotField extends Disposable {
    private _id: string;
    private _sourceColumnIndex: number;
    private _name: string;
    private _area: PivotFieldAreaType;
    private _aggregation?: AggregationType;
    private _sort: PivotSortOrder = PivotSortOrder.NONE;
    private _filter?: IPivotFilterCriteria;

    constructor(
        id: string,
        sourceColumnIndex: number,
        name: string,
        area: PivotFieldAreaType,
        aggregation?: AggregationType
    ) {
        super();
        this._id = id;
        this._sourceColumnIndex = sourceColumnIndex;
        this._name = name;
        this._area = area;
        this._aggregation = aggregation;

        // Default aggregation for value fields
        if (area === PivotFieldAreaType.VALUE && !aggregation) {
            this._aggregation = AggregationType.SUM;
        }
    }

    getId(): string {
        return this._id;
    }

    getSourceColumnIndex(): number {
        return this._sourceColumnIndex;
    }

    getName(): string {
        return this._name;
    }

    setName(name: string): void {
        this._name = name;
    }

    getArea(): PivotFieldAreaType {
        return this._area;
    }

    setArea(area: PivotFieldAreaType): void {
        this._area = area;
        // Set default aggregation for value fields
        if (area === PivotFieldAreaType.VALUE && !this._aggregation) {
            this._aggregation = AggregationType.SUM;
        }
    }

    getAggregation(): AggregationType | undefined {
        return this._aggregation;
    }

    setAggregation(aggregation: AggregationType): void {
        this._aggregation = aggregation;
    }

    getSort(): PivotSortOrder {
        return this._sort;
    }

    setSort(sort: PivotSortOrder): void {
        this._sort = sort;
    }

    getFilter(): IPivotFilterCriteria | undefined {
        return this._filter;
    }

    setFilter(filter: IPivotFilterCriteria | undefined): void {
        this._filter = filter;
    }

    /**
     * Convert to JSON for serialization
     */
    toJSON(): IPivotField {
        return {
            id: this._id,
            sourceColumnIndex: this._sourceColumnIndex,
            name: this._name,
            area: this._area,
            aggregation: this._aggregation,
            sort: this._sort,
            filter: this._filter,
        };
    }

    /**
     * Create PivotField from JSON
     */
    static fromJSON(json: IPivotField): PivotField {
        const field = new PivotField(
            json.id,
            json.sourceColumnIndex,
            json.name,
            json.area,
            json.aggregation
        );
        if (json.sort) {
            field.setSort(json.sort);
        }
        if (json.filter) {
            field.setFilter(json.filter);
        }
        return field;
    }

    /**
     * Clone the pivot field
     */
    clone(): PivotField {
        const field = new PivotField(
            this._id,
            this._sourceColumnIndex,
            this._name,
            this._area,
            this._aggregation
        );
        field.setSort(this._sort);
        if (this._filter) {
            field.setFilter({ ...this._filter });
        }
        return field;
    }
}
