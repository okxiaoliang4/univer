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

import type { closestCenter, DndContext, DragEndEvent, DragOverEvent, DragOverlay, DragStartEvent, KeyboardSensor, PointerSensor, type UniqueIdentifier, useSensor, useSensors } from '@dnd-kit/core';
import type { Workbook } from '@univerjs/core';
import type { PivotTable } from '@univerjs/sheets-pivot-table';
import { useDroppable } from '@dnd-kit/core';
import {
    arrayMove,
    SortableContext,
    sortableKeyboardCoordinates,
    useSortable,
    verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { IUniverInstanceService, LocaleService } from '@univerjs/core';
import { Button } from '@univerjs/design';
import { ConditionsDoubleIcon, GridIcon, MenuIcon, MoreDownIcon } from '@univerjs/icons';
import { PivotFieldAreaType } from '@univerjs/sheets-pivot-table';
import { useDependency } from '@univerjs/ui';
import { useCallback, useEffect, useState } from 'react';
import './PivotTableEditor.css';

interface IPivotTableEditorProps {
    pivotTable: PivotTable;
}

interface IFieldItem {
    id: string;
    name: string;
    columnIndex: number;
}

type FieldArea = PivotFieldAreaType | 'available';

interface IDraggedField {
    id: string;
    area: FieldArea;
}

// Field Chip Component
interface IFieldChipProps {
    fieldId: string;
    area: FieldArea;
    getFieldName: (id: string) => string;
    showRemove?: boolean;
    showAggregation?: boolean;
    disabled?: boolean;
    onRemove?: (fieldId: string, area: PivotFieldAreaType) => void;
    localeService: LocaleService;
}

const FieldChip: React.FC<IFieldChipProps> = ({
    fieldId,
    area,
    getFieldName,
    showRemove = false,
    showAggregation = false,
    disabled = false,
    onRemove,
    localeService,
}) => {
    const {
        attributes,
        listeners,
        setNodeRef,
        transform,
        transition,
        isDragging,
    } = useSortable({
        id: fieldId,
        disabled,
        data: {
            type: 'field',
            area,
            fieldId,
        },
    });

    const style = {
        transform: CSS.Transform.toString(transform),
        transition,
    };

    const className = [
        'univer-pivot-table-field-chip',
        isDragging ? 'dragging' : '',
        disabled ? 'in-use' : '',
    ].filter(Boolean).join(' ');

    return (
        <button
            ref={setNodeRef}
            style={style}
            className={className}
            type="button"
            disabled={disabled}
            {...attributes}
            {...listeners}
        >
            <span className="univer-pivot-table-field-drag-handle" aria-hidden="true">⋮⋮</span>
            <span>{getFieldName(fieldId)}</span>
            {showAggregation && (
                <span className="univer-pivot-table-aggregation-badge">SUM</span>
            )}
            {showRemove && onRemove && (
                <button
                    type="button"
                    className="univer-pivot-table-field-remove-btn"
                    onClick={(e) => {
                        e.stopPropagation();
                        onRemove(fieldId, area as PivotFieldAreaType);
                    }}
                    title={localeService.t('pivotTable.editor.remove')}
                    aria-label={localeService.t('pivotTable.editor.remove')}
                >
                    ×
                </button>
            )}
        </button>
    );
};

// Drop Zone Component
interface IDropZoneProps {
    id: string;
    area: PivotFieldAreaType;
    title: string;
    icon: React.ReactNode;
    fields: string[];
    getFieldName: (id: string) => string;
    showAggregation?: boolean;
    onRemoveField: (fieldId: string, area: PivotFieldAreaType) => void;
    localeService: LocaleService;
}

const DropZone: React.FC<IDropZoneProps> = ({
    id,
    area,
    title,
    icon,
    fields,
    getFieldName,
    showAggregation = false,
    onRemoveField,
    localeService,
}) => {
    const { setNodeRef, isOver } = useDroppable({
        id,
        data: {
            type: 'dropzone',
            area,
        },
    });

    const className = [
        'univer-pivot-table-drop-zone',
        isOver ? 'drag-over' : '',
    ].filter(Boolean).join(' ');

    return (
        <section
            ref={setNodeRef}
            className={className}
            aria-label={title}
        >
            <div className="univer-pivot-table-drop-zone-header">
                <span className="univer-pivot-table-drop-zone-icon" aria-hidden="true">{icon}</span>
                <h4 className="univer-pivot-table-drop-zone-title">{title}</h4>
            </div>
            <div className="univer-pivot-table-drop-zone-content">
                {fields.length === 0
                    ? (
                        <div className="univer-pivot-table-drop-zone-empty">
                            {localeService.t('pivotTable.editor.dragFieldsHere')}
                        </div>
                    )
                    : (
                        <SortableContext items={fields} strategy={verticalListSortingStrategy}>
                            {fields.map((fieldId) => (
                                <FieldChip
                                    key={fieldId}
                                    fieldId={fieldId}
                                    area={area}
                                    getFieldName={getFieldName}
                                    showRemove
                                    showAggregation={showAggregation}
                                    onRemove={onRemoveField}
                                    localeService={localeService}
                                />
                            ))}
                        </SortableContext>
                    )}
            </div>
        </section>
    );
};

// Available Fields List Component
interface IAvailableFieldsListProps {
    fields: IFieldItem[];
    usedFields: Set<string>;
    getFieldName: (id: string) => string;
    localeService: LocaleService;
    activeId: UniqueIdentifier | null;
}

const AvailableFieldsList: React.FC<IAvailableFieldsListProps> = ({
    fields,
    usedFields,
    getFieldName,
    localeService,
    activeId,
}) => {
    const { setNodeRef, isOver } = useDroppable({
        id: 'available-fields',
        data: {
            type: 'dropzone',
            area: 'available',
        },
    });

    const availableFieldIds = fields.map((f) => f.id);
    const className = [
        'univer-pivot-table-field-list',
        isOver ? 'drag-over' : '',
    ].filter(Boolean).join(' ');

    return (
        <ul
            ref={setNodeRef}
            className={className}
            aria-label={localeService.t('pivotTable.editor.availableFields')}
        >
            {fields.length === 0
                ? (
                    <li className="univer-pivot-table-field-list-empty">
                        {localeService.t('pivotTable.editor.dragFieldsHere')}
                    </li>
                )
                : (
                    <SortableContext items={availableFieldIds} strategy={verticalListSortingStrategy}>
                        {fields.map((field) => {
                            const isInUse = usedFields.has(field.id) && activeId !== field.id;
                            return (
                                <li key={field.id} style={{ display: 'inline' }}>
                                    <FieldChip
                                        fieldId={field.id}
                                        area="available"
                                        getFieldName={getFieldName}
                                        disabled={isInUse}
                                        localeService={localeService}
                                    />
                                </li>
                            );
                        })}
                    </SortableContext>
                )}
        </ul>
    );
};

export const PivotTableEditor: React.FC<IPivotTableEditorProps> = ({ pivotTable }) => {
    const localeService = useDependency(LocaleService);
    const univerInstanceService = useDependency(IUniverInstanceService);

    // Get available fields from source data
    const [availableFields, setAvailableFields] = useState<IFieldItem[]>([]);
    const [rowFields, setRowFields] = useState<string[]>([]);
    const [columnFields, setColumnFields] = useState<string[]>([]);
    const [valueFields, setValueFields] = useState<string[]>([]);
    const [filterFields, setFilterFields] = useState<string[]>([]);

    const [activeId, setActiveId] = useState<UniqueIdentifier | null>(null);
    const [draggedField, setDraggedField] = useState<IDraggedField | null>(null);

    const sensors = useSensors(
        useSensor(PointerSensor),
        useSensor(KeyboardSensor, {
            coordinateGetter: sortableKeyboardCoordinates,
        })
    );

    // Load initial data
    useEffect(() => {
        const sourceRangeInfo = pivotTable.getSourceRangeInfo();
        const workbook = univerInstanceService.getUnit(sourceRangeInfo.unitId) as Workbook;

        if (workbook) {
            const worksheet = workbook.getActiveSheet();
            if (worksheet) {
                const range = worksheet.getRange(sourceRangeInfo.range);
                const matrix = range.getMatrix();

                // Get headers from first row
                const headers: IFieldItem[] = [];
                const startCol = sourceRangeInfo.range.startColumn;
                const endCol = sourceRangeInfo.range.endColumn;

                for (let col = startCol; col <= endCol; col++) {
                    const cell = matrix.getValue(sourceRangeInfo.range.startRow, col);
                    const headerText = cell?.v?.toString() || `Column ${col + 1}`;
                    headers.push({
                        id: `field_${col}`,
                        name: headerText,
                        columnIndex: col,
                    });
                }

                setAvailableFields(headers);
            }
        }

        // Load current configuration
        setRowFields(pivotTable.getRowFields());
        setColumnFields(pivotTable.getColumnFields());
        setValueFields(pivotTable.getValueFields());
        setFilterFields(pivotTable.getFilterFields());
    }, [pivotTable, univerInstanceService]);

    // Get field name by ID
    const getFieldName = useCallback((fieldId: string): string => {
        const field = availableFields.find((f) => f.id === fieldId);
        return field?.name || fieldId;
    }, [availableFields]);

    // Get used fields
    const usedFields = new Set([
        ...rowFields,
        ...columnFields,
        ...valueFields,
        ...filterFields,
    ]);

    // Get field list by area
    const getFieldsByArea = useCallback((area: FieldArea): string[] => {
        switch (area) {
            case PivotFieldAreaType.ROW:
                return rowFields;
            case PivotFieldAreaType.COLUMN:
                return columnFields;
            case PivotFieldAreaType.VALUE:
                return valueFields;
            case PivotFieldAreaType.FILTER:
                return filterFields;
            case 'available':
                return availableFields.map((f) => f.id);
            default:
                return [];
        }
    }, [rowFields, columnFields, valueFields, filterFields, availableFields]);

    // Set fields by area
    const setFieldsByArea = useCallback((area: FieldArea, fields: string[]) => {
        switch (area) {
            case PivotFieldAreaType.ROW:
                setRowFields(fields);
                break;
            case PivotFieldAreaType.COLUMN:
                setColumnFields(fields);
                break;
            case PivotFieldAreaType.VALUE:
                setValueFields(fields);
                break;
            case PivotFieldAreaType.FILTER:
                setFilterFields(fields);
                break;
            case 'available':
                // Available fields are read-only
                break;
        }
    }, []);

    // Handle drag start
    const handleDragStart = useCallback((event: DragStartEvent) => {
        const { active } = event;
        setActiveId(active.id);

        const activeData = active.data.current;
        if (activeData?.type === 'field') {
            setDraggedField({
                id: active.id as string,
                area: activeData.area as FieldArea,
            });
        }
    }, []);

    // Handle drag over - for visual feedback only, actual reordering happens in dragEnd
    const handleDragOver = useCallback((_event: DragOverEvent) => {
        // Visual feedback is handled by dnd-kit automatically
    }, []);

    // Handle drag end
    const handleDragEnd = useCallback((event: DragEndEvent) => {
        const { active, over } = event;

        setActiveId(null);
        setDraggedField(null);

        if (!over) return;

        const activeData = active.data.current;
        const overData = over.data.current;

        if (!activeData || activeData.type !== 'field') return;

        const activeArea = activeData.area as FieldArea;
        const activeFieldId = active.id as string;

        // Handle sorting within the same container (handled by SortableContext)
        if (overData?.type === 'field' && activeArea === overData.area) {
            const fields = getFieldsByArea(activeArea);
            const oldIndex = fields.indexOf(activeFieldId);
            const newIndex = fields.indexOf(over.id as string);

            if (oldIndex !== newIndex && oldIndex !== -1 && newIndex !== -1) {
                const newFields = arrayMove(fields, oldIndex, newIndex);
                setFieldsByArea(activeArea, newFields);
            }
            return;
        }

        // Handle drop into dropzone (different area)
        if (overData?.type === 'dropzone') {
            const targetArea = overData.area as FieldArea;

            // If dropped in the same area, do nothing (sorting already handled above)
            if (activeArea === targetArea) {
                return;
            }

            // Remove from source area
            const sourceFields = getFieldsByArea(activeArea);
            const newSourceFields = sourceFields.filter((id) => id !== activeFieldId);
            setFieldsByArea(activeArea, newSourceFields);

            // Add to target area
            const targetFields = getFieldsByArea(targetArea);
            if (!targetFields.includes(activeFieldId)) {
                setFieldsByArea(targetArea, [...targetFields, activeFieldId]);
            }
        }
    }, [getFieldsByArea, setFieldsByArea]);

    const handleRemoveField = useCallback((
        fieldId: string,
        area: PivotFieldAreaType
    ) => {
        const fields = getFieldsByArea(area);
        setFieldsByArea(area, fields.filter((id) => id !== fieldId));
    }, [getFieldsByArea, setFieldsByArea]);

    const handleApply = useCallback(() => {
        // Update pivot table configuration
        pivotTable.setRowFields(rowFields);
        pivotTable.setColumnFields(columnFields);
        pivotTable.setValueFields(valueFields);
        pivotTable.setFilters(filterFields);

        // Here you would typically dispatch a command to update the configuration
        // commandService.executeCommand(UpdatePivotTableFieldsConfigCommand.id, { ... });
    }, [pivotTable, rowFields, columnFields, valueFields, filterFields]);

    // Render active drag overlay
    const renderDragOverlay = () => {
        if (!draggedField) return null;

        return (
            <DragOverlay>
                <div className="univer-pivot-table-field-chip dragging">
                    <span className="univer-pivot-table-field-drag-handle" aria-hidden="true">⋮⋮</span>
                    <span>{getFieldName(draggedField.id)}</span>
                </div>
            </DragOverlay>
        );
    };

    return (
        <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragStart={handleDragStart}
            onDragOver={handleDragOver}
            onDragEnd={handleDragEnd}
        >
            <div className="univer-pivot-table-editor">
                <div className="univer-pivot-table-editor-header">
                    <h3 className="univer-pivot-table-editor-title">
                        {localeService.t('pivotTable.editor.title')}
                    </h3>
                </div>

                <div className="univer-pivot-table-editor-content">
                    {/* Available Fields Section */}
                    <div className="univer-pivot-table-editor-section">
                        <h4 className="univer-pivot-table-editor-section-title">
                            {localeService.t('pivotTable.editor.availableFields')}
                        </h4>
                        <AvailableFieldsList
                            fields={availableFields}
                            usedFields={usedFields}
                            getFieldName={getFieldName}
                            localeService={localeService}
                            activeId={activeId}
                        />
                    </div>

                    {/* Drop Zones */}
                    <div className="univer-pivot-table-drop-zones-container">
                        <DropZone
                            id="filter-dropzone"
                            area={PivotFieldAreaType.FILTER}
                            title={localeService.t('pivotTable.editor.filters')}
                            icon={<ConditionsDoubleIcon />}
                            fields={filterFields}
                            getFieldName={getFieldName}
                            onRemoveField={handleRemoveField}
                            localeService={localeService}
                        />
                        <DropZone
                            id="column-dropzone"
                            area={PivotFieldAreaType.COLUMN}
                            title={localeService.t('pivotTable.editor.columns')}
                            icon={<GridIcon />}
                            fields={columnFields}
                            getFieldName={getFieldName}
                            onRemoveField={handleRemoveField}
                            localeService={localeService}
                        />
                        <DropZone
                            id="row-dropzone"
                            area={PivotFieldAreaType.ROW}
                            title={localeService.t('pivotTable.editor.rows')}
                            icon={<MenuIcon />}
                            fields={rowFields}
                            getFieldName={getFieldName}
                            onRemoveField={handleRemoveField}
                            localeService={localeService}
                        />
                        <DropZone
                            id="value-dropzone"
                            area={PivotFieldAreaType.VALUE}
                            title={localeService.t('pivotTable.editor.values')}
                            icon={<MoreDownIcon />}
                            fields={valueFields}
                            getFieldName={getFieldName}
                            showAggregation
                            onRemoveField={handleRemoveField}
                            localeService={localeService}
                        />
                    </div>
                </div>

                <div className="univer-pivot-table-editor-footer">
                    <Button onClick={() => { /* Handle cancel */ }}>
                        {localeService.t('pivotTable.dialog.cancel')}
                    </Button>
                    <Button variant="primary" onClick={handleApply}>
                        {localeService.t('pivotTable.dialog.confirm')}
                    </Button>
                </div>
            </div>
            {renderDragOverlay()}
        </DndContext>
    );
};
