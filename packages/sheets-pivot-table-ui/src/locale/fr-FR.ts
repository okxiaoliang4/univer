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

import type enUS from './en-US';

const locale: typeof enUS = {
    pivotTable: {
        menu: {
            insert: 'Tableau croisé dynamique',
        },
        dialog: {
            createTitle: 'Créer un tableau croisé dynamique',
            sourceRangeLabel: 'Sélectionner la plage de données source',
            targetRangeLabel: 'Choisir où placer le tableau croisé dynamique',
            targetRangeTypeNew: 'Nouvelle feuille',
            targetRangeTypeExisting: 'Feuille existante',
            sourceRangeSingleCellError: 'La plage source doit contenir plus d\'une cellule',
            sourceRangeSingleRowError: 'La plage source doit contenir au moins deux lignes (en-tête et données)',
            sourceRangeWithMergeError: 'La plage source ne peut pas chevaucher des cellules fusionnées',
            targetRangeSingleCellError: 'L\'emplacement cible doit être une seule cellule',
            invalidWorksheet: 'Feuille de calcul non valide',
            cancel: 'Annuler',
            confirm: 'OK',
        },
        editor: {
            title: 'Champs du tableau croisé dynamique',
            availableFields: 'Champs disponibles',
            filters: 'Filtres',
            columns: 'Colonnes',
            rows: 'Lignes',
            values: 'Valeurs',
            dragFieldsHere: 'Faites glisser les champs ici',
            remove: 'Supprimer',
            add: 'Ajouter',
            aggregationMethodLabel: 'Méthode d\'agrégation',
            aggregationType: {
                sum: 'Somme',
                count: 'Compter',
                average: 'Moyenne',
                min: 'Minimum',
                max: 'Maximum',
            },
            containerLabel: 'Colonne {0}',
            containerLabels: {
                sourceFields: 'Champs disponibles',
                filterFields: 'Filtres',
                columnFields: 'Colonnes',
                rowFields: 'Lignes',
                valueFields: 'Valeurs',
            },
        },
        panel: {
            sourceRangeLabel: 'Plage source:',
            fieldConfigurationLabel: 'Configuration des champs:',
            valuePositionLabel: 'Position de la valeur:',
            valuePositionRow: 'Ligne',
            valuePositionColumn: 'Colonne',
        },
    },
};

export default locale;
