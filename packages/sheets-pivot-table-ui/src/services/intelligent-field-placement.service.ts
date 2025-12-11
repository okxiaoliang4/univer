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

import type { ICellData, IObjectMatrixPrimitiveType, Nullable } from '@univerjs/core';
import type { AggregationType, IFieldsConfig } from '@univerjs/sheets-pivot-table';
import type { FieldPlacementArea } from './field-placement-preference.service';
import { AggregationType as AggregationTypeEnum } from '@univerjs/sheets-pivot-table';
import { FieldPlacementPreferenceService } from './field-placement-preference.service';
import { FieldTypeDetectionService } from './field-type-detection.service';
import { FuzzyMatchingService } from './fuzzy-matching.service';

/**
 * Service that combines data type detection, fuzzy matching, and user preferences
 * to determine intelligent field placement
 */
export class IntelligentFieldPlacementService {
    private _fieldTypeDetection: FieldTypeDetectionService;
    private _fuzzyMatching: FuzzyMatchingService;
    private _preferenceService: FieldPlacementPreferenceService;

    constructor() {
        this._fieldTypeDetection = new FieldTypeDetectionService();
        this._fuzzyMatching = new FuzzyMatchingService();
        this._preferenceService = new FieldPlacementPreferenceService();
    }

    /**
     * Determine the target area for a field using intelligent placement logic
     * Priority order: (1) user preference, (2) fuzzy matching, (3) data type detection
     *
     * @param fieldName - The name of the field
     * @param sourceColumnIndex - The source column index
     * @param sourceData - The source data matrix
     * @returns The target area and default aggregation (if valueFields)
     */
    determinePlacement(
        fieldName: string,
        sourceColumnIndex: number,
        sourceData: IObjectMatrixPrimitiveType<Nullable<ICellData>>
    ): {
        area: FieldPlacementArea;
        aggregation?: AggregationType;
    } {
        // Priority 1: Check user preference
        const userPreference = this._preferenceService.getPreference(fieldName);
        if (userPreference) {
            return {
                area: userPreference,
                aggregation: userPreference === 'valueFields' ? AggregationTypeEnum.SUM : undefined,
            };
        }

        // Priority 2: Check fuzzy matching rules
        const fuzzyMatch = this._fuzzyMatching.match(fieldName);
        if (fuzzyMatch) {
            return {
                area: fuzzyMatch,
                aggregation: undefined,
            };
        }

        // Priority 3: Use data type detection
        const fieldType = this._fieldTypeDetection.detectFieldType(sourceData, sourceColumnIndex);
        if (fieldType === 'numeric') {
            return {
                area: 'valueFields',
                aggregation: AggregationTypeEnum.SUM,
            };
        }

        // Default: text fields go to rowFields
        return {
            area: 'rowFields',
            aggregation: undefined,
        };
    }

    /**
     * Update user preference when a field is moved
     */
    updatePreference(fieldsConfig: IFieldsConfig): void {
        this._preferenceService.setPreference(fieldsConfig);
    }

    /**
     * Remove preference when a field is removed from all areas
     */
    removePreference(fieldName: string): void {
        this._preferenceService.removePreference(fieldName);
    }

    /**
     * Get the field type detection service (for direct access if needed)
     */
    getFieldTypeDetection(): FieldTypeDetectionService {
        return this._fieldTypeDetection;
    }

    /**
     * Get the fuzzy matching service (for direct access if needed)
     */
    getFuzzyMatching(): FuzzyMatchingService {
        return this._fuzzyMatching;
    }

    /**
     * Get the preference service (for direct access if needed)
     */
    getPreferenceService(): FieldPlacementPreferenceService {
        return this._preferenceService;
    }
}
