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

import type { FieldPlacementArea } from './field-placement-preference.service';

export interface IFuzzyMatchingRule {
    match: string; // Case-insensitive pattern to match
    type: 'row' | 'column'; // Target area type
}

/**
 * Default fuzzy matching rules for common field name patterns
 */
const DEFAULT_FUZZY_RULES: IFuzzyMatchingRule[] = [
    { match: 'userId', type: 'row' },
    { match: 'user id', type: 'row' },
    { match: 'id', type: 'row' },
    { match: 'name', type: 'row' },
    { match: 'category', type: 'row' },
    { match: 'type', type: 'row' },
    { match: 'status', type: 'row' },
    { match: 'date', type: 'row' },
    { match: 'month', type: 'column' },
    { match: 'year', type: 'column' },
    { match: 'quarter', type: 'column' },
];

/**
 * Service for fuzzy matching field names to placement rules
 */
export class FuzzyMatchingService {
    private _rules: IFuzzyMatchingRule[] = [...DEFAULT_FUZZY_RULES];

    /**
     * Get all fuzzy matching rules
     */
    getRules(): IFuzzyMatchingRule[] {
        return [...this._rules];
    }

    /**
     * Add a fuzzy matching rule
     */
    addRule(rule: IFuzzyMatchingRule): void {
        // Check if rule already exists
        const exists = this._rules.some(
            (r) => r.match.toLowerCase() === rule.match.toLowerCase() && r.type === rule.type
        );
        if (!exists) {
            this._rules.push(rule);
        }
    }

    /**
     * Remove a fuzzy matching rule
     */
    removeRule(match: string, type: 'row' | 'column'): void {
        this._rules = this._rules.filter(
            (r) => !(r.match.toLowerCase() === match.toLowerCase() && r.type === type)
        );
    }

    /**
     * Match a field name against fuzzy rules
     * @param fieldName - The field name to match
     * @returns The target area type ('rowFields' or 'columnFields') if matched, null otherwise
     */
    match(fieldName: string): FieldPlacementArea | null {
        const lowerFieldName = fieldName.toLowerCase();

        for (const rule of this._rules) {
            const lowerMatch = rule.match.toLowerCase();

            // Check for exact match or substring match
            if (lowerFieldName === lowerMatch || lowerFieldName.includes(lowerMatch)) {
                return rule.type === 'row' ? 'rowFields' : 'columnFields';
            }
        }

        return null;
    }

    /**
     * Reset to default rules
     */
    resetToDefaults(): void {
        this._rules = [...DEFAULT_FUZZY_RULES];
    }
}
