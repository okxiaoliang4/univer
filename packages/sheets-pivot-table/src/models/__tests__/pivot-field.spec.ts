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

import { afterEach, describe, expect, it } from 'vitest';
import { PivotField } from '../../model/pivot-field';
import { AggregationType, PivotFieldAreaType, PivotSortOrder } from '../../types/enum';

describe('PivotField', () => {
    let field: PivotField;

    afterEach(() => {
        field?.dispose();
    });

    describe('constructor', () => {
        it('should create a pivot field with basic properties', () => {
            field = new PivotField('field1', 0, 'Category', PivotFieldAreaType.ROW);

            expect(field.getId()).toBe('field1');
            expect(field.getSourceColumnIndex()).toBe(0);
            expect(field.getName()).toBe('Category');
            expect(field.getArea()).toBe(PivotFieldAreaType.ROW);
        });

        it('should set default aggregation for value fields', () => {
            field = new PivotField('field1', 0, 'Amount', PivotFieldAreaType.VALUE);

            expect(field.getAggregation()).toBe(AggregationType.SUM);
        });

        it('should use provided aggregation for value fields', () => {
            field = new PivotField('field1', 0, 'Amount', PivotFieldAreaType.VALUE, AggregationType.AVERAGE);

            expect(field.getAggregation()).toBe(AggregationType.AVERAGE);
        });

        it('should not set aggregation for non-value fields', () => {
            field = new PivotField('field1', 0, 'Category', PivotFieldAreaType.ROW);

            expect(field.getAggregation()).toBeUndefined();
        });
    });

    describe('setters and getters', () => {
        it('should update field name', () => {
            field = new PivotField('field1', 0, 'Category', PivotFieldAreaType.ROW);

            field.setName('Updated Category');

            expect(field.getName()).toBe('Updated Category');
        });

        it('should update field area', () => {
            field = new PivotField('field1', 0, 'Category', PivotFieldAreaType.ROW);

            field.setArea(PivotFieldAreaType.COLUMN);

            expect(field.getArea()).toBe(PivotFieldAreaType.COLUMN);
        });

        it('should set default aggregation when changing to value area', () => {
            field = new PivotField('field1', 0, 'Amount', PivotFieldAreaType.ROW);

            field.setArea(PivotFieldAreaType.VALUE);

            expect(field.getAggregation()).toBe(AggregationType.SUM);
        });

        it('should update aggregation type', () => {
            field = new PivotField('field1', 0, 'Amount', PivotFieldAreaType.VALUE);

            field.setAggregation(AggregationType.COUNT);

            expect(field.getAggregation()).toBe(AggregationType.COUNT);
        });

        it('should update sort order', () => {
            field = new PivotField('field1', 0, 'Category', PivotFieldAreaType.ROW);

            field.setSort(PivotSortOrder.ASC);

            expect(field.getSort()).toBe(PivotSortOrder.ASC);
        });

        it('should update filter criteria', () => {
            field = new PivotField('field1', 0, 'Category', PivotFieldAreaType.ROW);

            const filter = {
                type: 'value' as const,
                values: ['A', 'B'],
            };

            field.setFilter(filter);

            expect(field.getFilter()).toEqual(filter);
        });

        it('should clear filter criteria', () => {
            field = new PivotField('field1', 0, 'Category', PivotFieldAreaType.ROW);

            field.setFilter({
                type: 'value',
                values: ['A'],
            });

            field.setFilter(undefined);

            expect(field.getFilter()).toBeUndefined();
        });
    });

    describe('toJSON', () => {
        it('should serialize to JSON', () => {
            field = new PivotField('field1', 0, 'Category', PivotFieldAreaType.ROW, AggregationType.SUM);
            field.setSort(PivotSortOrder.ASC);
            field.setFilter({
                type: 'value',
                values: ['A', 'B'],
            });

            const json = field.toJSON();

            expect(json).toEqual({
                id: 'field1',
                sourceColumnIndex: 0,
                name: 'Category',
                area: PivotFieldAreaType.ROW,
                aggregation: AggregationType.SUM,
                sort: PivotSortOrder.ASC,
                filter: {
                    type: 'value',
                    values: ['A', 'B'],
                },
            });
        });
    });

    describe('fromJSON', () => {
        it('should deserialize from JSON', () => {
            const json = {
                id: 'field1',
                sourceColumnIndex: 0,
                name: 'Category',
                area: PivotFieldAreaType.ROW,
                aggregation: AggregationType.SUM,
                sort: PivotSortOrder.DESC,
                filter: {
                    type: 'value' as const,
                    values: ['A', 'B'],
                },
            };

            field = PivotField.fromJSON(json);

            expect(field.getId()).toBe('field1');
            expect(field.getName()).toBe('Category');
            expect(field.getArea()).toBe(PivotFieldAreaType.ROW);
            expect(field.getAggregation()).toBe(AggregationType.SUM);
            expect(field.getSort()).toBe(PivotSortOrder.DESC);
            expect(field.getFilter()).toEqual({
                type: 'value',
                values: ['A', 'B'],
            });
        });
    });

    describe('clone', () => {
        it('should create a deep copy of the field', () => {
            field = new PivotField('field1', 0, 'Category', PivotFieldAreaType.ROW);
            field.setSort(PivotSortOrder.ASC);
            field.setFilter({
                type: 'value',
                values: ['A', 'B'],
            });

            const clonedField = field.clone();

            expect(clonedField.getId()).toBe(field.getId());
            expect(clonedField.getName()).toBe(field.getName());
            expect(clonedField.getArea()).toBe(field.getArea());
            expect(clonedField.getSort()).toBe(field.getSort());
            expect(clonedField.getFilter()).toEqual(field.getFilter());

            // Verify it's a different instance
            expect(clonedField).not.toBe(field);

            clonedField.dispose();
        });
    });
});
