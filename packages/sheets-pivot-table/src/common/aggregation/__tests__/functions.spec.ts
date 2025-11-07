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

import { describe, expect, it } from 'vitest';
import { AggregationType } from '../../../types/enum';
import { AverageAggregator, CountAggregator, createAggregator, MaxAggregator, MinAggregator, SumAggregator } from '../functions';

describe('Aggregation Functions', () => {
    describe('SumAggregator', () => {
        it('should sum numeric values', () => {
            const aggregator = new SumAggregator();
            aggregator.init();
            aggregator.addValue({ v: 10 });
            aggregator.addValue({ v: 20 });
            aggregator.addValue({ v: 30 });

            expect(aggregator.getResult()).toBe(60);
        });

        it('should ignore non-numeric values', () => {
            const aggregator = new SumAggregator();
            aggregator.init();
            aggregator.addValue({ v: 10 });
            aggregator.addValue({ v: 'text' });
            aggregator.addValue({ v: 20 });

            expect(aggregator.getResult()).toBe(30);
        });

        it('should return null when no numeric values', () => {
            const aggregator = new SumAggregator();
            aggregator.init();
            aggregator.addValue({ v: 'text' });
            aggregator.addValue({ v: 'more text' });

            expect(aggregator.getResult()).toBeNull();
        });

        it('should handle empty cells', () => {
            const aggregator = new SumAggregator();
            aggregator.init();
            aggregator.addValue(null);
            aggregator.addValue(undefined);
            aggregator.addValue({ v: 10 });

            expect(aggregator.getResult()).toBe(10);
        });
    });

    describe('CountAggregator', () => {
        it('should count non-empty cells', () => {
            const aggregator = new CountAggregator();
            aggregator.init();
            aggregator.addValue({ v: 10 });
            aggregator.addValue({ v: 'text' });
            aggregator.addValue({ v: true });

            expect(aggregator.getResult()).toBe(3);
        });

        it('should not count empty cells', () => {
            const aggregator = new CountAggregator();
            aggregator.init();
            aggregator.addValue({ v: 10 });
            aggregator.addValue(null);
            aggregator.addValue(undefined);
            aggregator.addValue({ v: '' });
            aggregator.addValue({ v: 20 });

            expect(aggregator.getResult()).toBe(2);
        });

        it('should return 0 for all empty cells', () => {
            const aggregator = new CountAggregator();
            aggregator.init();
            aggregator.addValue(null);
            aggregator.addValue(undefined);

            expect(aggregator.getResult()).toBe(0);
        });
    });

    describe('AverageAggregator', () => {
        it('should calculate average of numeric values', () => {
            const aggregator = new AverageAggregator();
            aggregator.init();
            aggregator.addValue({ v: 10 });
            aggregator.addValue({ v: 20 });
            aggregator.addValue({ v: 30 });

            expect(aggregator.getResult()).toBe(20);
        });

        it('should ignore non-numeric values', () => {
            const aggregator = new AverageAggregator();
            aggregator.init();
            aggregator.addValue({ v: 10 });
            aggregator.addValue({ v: 'text' });
            aggregator.addValue({ v: 30 });

            expect(aggregator.getResult()).toBe(20);
        });

        it('should return null when no numeric values', () => {
            const aggregator = new AverageAggregator();
            aggregator.init();
            aggregator.addValue({ v: 'text' });

            expect(aggregator.getResult()).toBeNull();
        });
    });

    describe('MinAggregator', () => {
        it('should find minimum numeric value', () => {
            const aggregator = new MinAggregator();
            aggregator.init();
            aggregator.addValue({ v: 30 });
            aggregator.addValue({ v: 10 });
            aggregator.addValue({ v: 20 });

            expect(aggregator.getResult()).toBe(10);
        });

        it('should ignore non-numeric values', () => {
            const aggregator = new MinAggregator();
            aggregator.init();
            aggregator.addValue({ v: 30 });
            aggregator.addValue({ v: 'text' });
            aggregator.addValue({ v: 10 });

            expect(aggregator.getResult()).toBe(10);
        });

        it('should return null when no numeric values', () => {
            const aggregator = new MinAggregator();
            aggregator.init();
            aggregator.addValue({ v: 'text' });

            expect(aggregator.getResult()).toBeNull();
        });

        it('should handle negative numbers', () => {
            const aggregator = new MinAggregator();
            aggregator.init();
            aggregator.addValue({ v: 10 });
            aggregator.addValue({ v: -5 });
            aggregator.addValue({ v: 20 });

            expect(aggregator.getResult()).toBe(-5);
        });
    });

    describe('MaxAggregator', () => {
        it('should find maximum numeric value', () => {
            const aggregator = new MaxAggregator();
            aggregator.init();
            aggregator.addValue({ v: 10 });
            aggregator.addValue({ v: 30 });
            aggregator.addValue({ v: 20 });

            expect(aggregator.getResult()).toBe(30);
        });

        it('should ignore non-numeric values', () => {
            const aggregator = new MaxAggregator();
            aggregator.init();
            aggregator.addValue({ v: 10 });
            aggregator.addValue({ v: 'text' });
            aggregator.addValue({ v: 30 });

            expect(aggregator.getResult()).toBe(30);
        });

        it('should return null when no numeric values', () => {
            const aggregator = new MaxAggregator();
            aggregator.init();
            aggregator.addValue({ v: 'text' });

            expect(aggregator.getResult()).toBeNull();
        });

        it('should handle negative numbers', () => {
            const aggregator = new MaxAggregator();
            aggregator.init();
            aggregator.addValue({ v: -10 });
            aggregator.addValue({ v: -5 });
            aggregator.addValue({ v: -20 });

            expect(aggregator.getResult()).toBe(-5);
        });
    });

    describe('createAggregator', () => {
        it('should create SUM aggregator', () => {
            const aggregator = createAggregator(AggregationType.SUM);
            expect(aggregator).toBeInstanceOf(SumAggregator);
        });

        it('should create COUNT aggregator', () => {
            const aggregator = createAggregator(AggregationType.COUNT);
            expect(aggregator).toBeInstanceOf(CountAggregator);
        });

        it('should create AVERAGE aggregator', () => {
            const aggregator = createAggregator(AggregationType.AVERAGE);
            expect(aggregator).toBeInstanceOf(AverageAggregator);
        });

        it('should create MIN aggregator', () => {
            const aggregator = createAggregator(AggregationType.MIN);
            expect(aggregator).toBeInstanceOf(MinAggregator);
        });

        it('should create MAX aggregator', () => {
            const aggregator = createAggregator(AggregationType.MAX);
            expect(aggregator).toBeInstanceOf(MaxAggregator);
        });

        it('should throw error for unknown aggregation type', () => {
            expect(() => {
                createAggregator('unknown' as any);
            }).toThrow();
        });
    });

    describe('reset functionality', () => {
        it('should reset aggregator state', () => {
            const aggregator = new SumAggregator();
            aggregator.init();
            aggregator.addValue({ v: 10 });
            aggregator.addValue({ v: 20 });

            expect(aggregator.getResult()).toBe(30);

            aggregator.reset();
            aggregator.addValue({ v: 5 });

            expect(aggregator.getResult()).toBe(5);
        });
    });
});
