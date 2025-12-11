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

import type { IFieldsConfig } from '@univerjs/sheets-pivot-table';

export type FieldPlacementArea = 'rowFields' | 'columnFields' | 'valueFields' | 'filterFields';

export interface IFieldPlacementPreference {
    fieldName: string;
    area: FieldPlacementArea;
}

const STORAGE_KEY = 'univer:pivot-table:field-placement-preferences';

/**
 * Service for managing user field placement preferences in browser storage
 */
export class FieldPlacementPreferenceService {
    /**
     * Get all stored preferences
     */
    getPreferences(): Map<string, FieldPlacementArea> {
        try {
            const stored = localStorage.getItem(STORAGE_KEY);
            if (!stored) {
                return new Map();
            }

            const preferences: IFieldPlacementPreference[] = JSON.parse(stored);
            const map = new Map<string, FieldPlacementArea>();

            for (const pref of preferences) {
                // Use lowercase for case-insensitive matching
                map.set(pref.fieldName.toLowerCase(), pref.area);
            }

            return map;
        } catch {
            return new Map();
        }
    }

    /**
     * Get preference for a specific field
     */
    getPreference(fieldName: string): FieldPlacementArea | null {
        const preferences = this.getPreferences();
        return preferences.get(fieldName.toLowerCase()) || null;
    }

    /**
     * Set preference for a field
     */
    setPreference(fieldsConfig: IFieldsConfig): void {
        try {
            const preferences = this.getPreferences();
            for (const field of fieldsConfig.valueFields) {
                preferences.set(field.name.toLowerCase(), 'valueFields');
            }
            for (const field of fieldsConfig.rowFields) {
                preferences.set(field.name.toLowerCase(), 'rowFields');
            }
            for (const field of fieldsConfig.columnFields) {
                preferences.set(field.name.toLowerCase(), 'columnFields');
            }
            for (const field of fieldsConfig.filterFields) {
                preferences.set(field.name.toLowerCase(), 'filterFields');
            }

            // Convert back to array format
            const array: IFieldPlacementPreference[] = [];
            for (const [name, area] of preferences.entries()) {
                array.push({ fieldName: name, area });
            }

            localStorage.setItem(STORAGE_KEY, JSON.stringify(array));
        } catch {
            // Ignore storage errors (e.g., private browsing mode)
        }
    }

    /**
     * Remove preference for a field
     */
    removePreference(fieldName: string): void {
        try {
            const preferences = this.getPreferences();
            preferences.delete(fieldName.toLowerCase());

            // Convert back to array format
            const array: IFieldPlacementPreference[] = [];
            for (const [name, area] of preferences.entries()) {
                array.push({ fieldName: name, area });
            }

            localStorage.setItem(STORAGE_KEY, JSON.stringify(array));
        } catch {
            // Ignore storage errors
        }
    }

    /**
     * Clear all preferences
     */
    clearPreferences(): void {
        try {
            localStorage.removeItem(STORAGE_KEY);
        } catch {
            // Ignore storage errors
        }
    }
}
