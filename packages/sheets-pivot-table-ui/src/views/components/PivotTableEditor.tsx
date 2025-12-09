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

import type {
    CancelDrop,
    CollisionDetection,
    DragEndEvent,
    DraggableSyntheticListeners,
    DropAnimation,
    KeyboardCoordinateGetter,
    Modifiers,
    UniqueIdentifier,
} from '@dnd-kit/core';
import type {
    AnimateLayoutChanges,
    SortingStrategy,
} from '@dnd-kit/sortable';
import type { Transform } from '@dnd-kit/utilities';
import type { IFieldsConfig, IPivotField, IUpdatePivotTableFieldsCommandParams, PivotTable } from '@univerjs/sheets-pivot-table';
import type React from 'react';
import type { IFieldItemsContainerProps } from './FieldItemsContainer';
import {
    closestCenter,
    defaultDropAnimationSideEffects,
    DndContext,
    DragOverlay,
    getFirstCollision,
    KeyboardSensor,
    MeasuringStrategy,
    MouseSensor,
    pointerWithin,
    rectIntersection,
    TouchSensor,
    useSensor,
    useSensors,
} from '@dnd-kit/core';
import {
    arrayMove,
    defaultAnimateLayoutChanges,
    SortableContext,
    useSortable,
    verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { generateRandomId, ICommandService, LocaleService } from '@univerjs/core';
import { Button, Checkbox, Dropdown, Select, SelectList } from '@univerjs/design';
import { AggregationType, UpdatePivotTableFieldsCommand } from '@univerjs/sheets-pivot-table';
import { useDependency, useObservable } from '@univerjs/ui';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { coordinateGetter as multipleContainersCoordinateGetter } from '../../common/multiple-containers-keyboard-coordinates';
import { IntelligentFieldPlacementService } from '../../services/intelligent-field-placement.service';
import { FieldItem } from './FieldItem';
import { FieldItemsContainer } from './FieldItemsContainer';
import { FieldRender } from './FieldRender';

const animateLayoutChanges: AnimateLayoutChanges = (args) =>
    defaultAnimateLayoutChanges({ ...args, wasDragging: true });

function DroppableContainer({
    children,
    columns = 1,
    disabled,
    id,
    items,
    style,
    ...props
}: IFieldItemsContainerProps & {
    disabled?: boolean;
    id: UniqueIdentifier;
    items: UniqueIdentifier[];
    style?: React.CSSProperties;
}) {
    const {
        active,
        attributes,
        isDragging,
        listeners,
        over,
        setNodeRef,
        transition,
        transform,
    } = useSortable({
        id,
        data: {
            type: 'container',
            children: items,
        },
        animateLayoutChanges,
    });
    const isOverContainer = over
        ? (id === over.id && active?.data.current?.type !== 'container') ||
    items.includes(over.id)
        : false;

    return (
        <FieldItemsContainer
            className="flex-1"
            ref={disabled ? undefined : setNodeRef}
            style={{
                ...style,
                transition,
                transform: CSS.Translate.toString(transform),
                opacity: isDragging ? 0.5 : undefined,
            }}
            hover={isOverContainer}
            handleProps={{
                ...attributes,
                ...listeners,
            }}
            columns={columns}
            {...props}
        >
            {children}
        </FieldItemsContainer>
    );
}

const dropAnimation: DropAnimation = {
    sideEffects: defaultDropAnimationSideEffects({
        styles: {
            active: {
                opacity: '0.5',
            },
        },
    }),
};

const defaultGetItemStyles = () => ({});
const defaultWrapperStyle = () => ({});

interface IPivotTableEditorProps {
    pivotTable: PivotTable;
    adjustScale?: boolean;
    cancelDrop?: CancelDrop;
    columns?: number;
    containerStyle?: React.CSSProperties;
    coordinateGetter?: KeyboardCoordinateGetter;
    getItemStyles?(args: {
        value: UniqueIdentifier;
        index: number;
        overIndex: number;
        isDragging: boolean;
        containerId: UniqueIdentifier;
        isSorting: boolean;
        isDragOverlay: boolean;
    }): React.CSSProperties;
    wrapperStyle?(args: { index: number }): React.CSSProperties;
    handle?: boolean;
    renderItem?(): React.ReactElement;
    strategy?: SortingStrategy;
    modifiers?: Modifiers;
    minimal?: boolean;
    trashable?: boolean;
    scrollable?: boolean;
    vertical?: boolean;
}

export function PivotTableEditor({
    pivotTable,
    adjustScale = false,
    cancelDrop,
    columns,
    handle = true,
    containerStyle,
    coordinateGetter = multipleContainersCoordinateGetter,
    getItemStyles = defaultGetItemStyles,
    wrapperStyle = defaultWrapperStyle,
    minimal = false,
    modifiers,
    renderItem,
    strategy = verticalListSortingStrategy,
    scrollable,
}: IPivotTableEditorProps) {
    // Get current fields from pivotTable observables
    // const sourceFields = useObservable(pivotTable.sourceFields$, pivotTable.getSourceFields());
    const valueFields = useObservable(pivotTable.valueFields$, pivotTable.getValueFields());
    const sourceFields = useObservable(pivotTable.sourceFields$, pivotTable.getSourceFields());
    const rowFields = useObservable(pivotTable.rowFields$, pivotTable.getRowFields());
    const columnFields = useObservable(pivotTable.columnFields$, pivotTable.getColumnFields());
    const filterFields = useObservable(pivotTable.filterFields$, pivotTable.getFilterFields());

    const commandService = useDependency(ICommandService);
    const localeService = useDependency(LocaleService);
    const intelligentPlacementService = useDependency(IntelligentFieldPlacementService);

    // Get source data for field type detection
    const sourceData = useMemo(() => pivotTable.getSourceData(), [pivotTable]);

    // Helper function to check if a field is already added to any area
    const isFieldAdded = useCallback((sourceColumnIndex: number): boolean => {
        return (
            rowFields.some((f) => f.sourceColumnIndex === sourceColumnIndex) ||
            columnFields.some((f) => f.sourceColumnIndex === sourceColumnIndex) ||
            valueFields.some((f) => f.sourceColumnIndex === sourceColumnIndex) ||
            filterFields.some((f) => f.sourceColumnIndex === sourceColumnIndex)
        );
    }, [rowFields, columnFields, valueFields, filterFields]);

    // Helper function to add a field with intelligent placement
    const addFieldWithIntelligentPlacement = useCallback((field: IPivotField) => {
        const placement = intelligentPlacementService.determinePlacement(
            field.name,
            field.sourceColumnIndex,
            sourceData
        );

        const targetCellInfo = pivotTable.getTargetCellInfo();
        let newValueFields = [...valueFields];
        let newRowFields = [...rowFields];
        let newColumnFields = [...columnFields];
        let newFilterFields = [...filterFields];

        const newField: IPivotField = {
            ...field,
            id: `${generateRandomId(6)}_${field.id}`,
            aggregation: placement.aggregation,
        };

        // Remove from other areas if it exists (exclusive rule for rowFields/columnFields)
        if (placement.area === 'rowFields' || placement.area === 'columnFields') {
            newColumnFields = newColumnFields.filter((f) => f.sourceColumnIndex !== field.sourceColumnIndex);
            newRowFields = newRowFields.filter((f) => f.sourceColumnIndex !== field.sourceColumnIndex);
        }

        // Add to target area
        if (placement.area === 'valueFields') {
            newValueFields = [...newValueFields, newField];
        } else if (placement.area === 'rowFields') {
            newRowFields = [...newRowFields, newField];
        } else if (placement.area === 'columnFields') {
            newColumnFields = [...newColumnFields, newField];
        } else if (placement.area === 'filterFields') {
            newFilterFields = [...newFilterFields, newField];
        }

        commandService.executeCommand(UpdatePivotTableFieldsCommand.id, {
            unitId: targetCellInfo.unitId,
            subUnitId: targetCellInfo.subUnitId,
            pivotTableId: pivotTable.getId(),
            fieldsConfig: {
                valueFields: newValueFields,
                rowFields: newRowFields,
                columnFields: newColumnFields,
                filterFields: newFilterFields,
                valuePosition: pivotTable.getValuePosition(),
            } satisfies IFieldsConfig,
        } satisfies IUpdatePivotTableFieldsCommandParams);
    }, [intelligentPlacementService, sourceData, pivotTable, commandService, valueFields, rowFields, columnFields, filterFields]);

    // Helper function to remove a field from all areas
    const removeFieldFromAllAreas = useCallback((field: IPivotField) => {
        const targetCellInfo = pivotTable.getTargetCellInfo();

        commandService.executeCommand(UpdatePivotTableFieldsCommand.id, {
            unitId: targetCellInfo.unitId,
            subUnitId: targetCellInfo.subUnitId,
            pivotTableId: pivotTable.getId(),
            fieldsConfig: {
                valueFields: valueFields.filter((f) => f.sourceColumnIndex !== field.sourceColumnIndex),
                rowFields: rowFields.filter((f) => f.sourceColumnIndex !== field.sourceColumnIndex),
                columnFields: columnFields.filter((f) => f.sourceColumnIndex !== field.sourceColumnIndex),
                filterFields: filterFields.filter((f) => f.sourceColumnIndex !== field.sourceColumnIndex),
                valuePosition: pivotTable.getValuePosition(),
            } satisfies IFieldsConfig,
        } satisfies IUpdatePivotTableFieldsCommandParams);
    }, [intelligentPlacementService, pivotTable, commandService, valueFields, rowFields, columnFields, filterFields]);

    // Helper function to add a field to a specific area
    const addFieldToArea = useCallback((field: IPivotField, targetArea: 'rowFields' | 'columnFields' | 'valueFields' | 'filterFields') => {
        const targetCellInfo = pivotTable.getTargetCellInfo();
        let newValueFields = [...valueFields];
        let newRowFields = [...rowFields];
        let newColumnFields = [...columnFields];
        let newFilterFields = [...filterFields];

        const newField: IPivotField = {
            ...field,
            id: `${generateRandomId(6)}_${field.id}`,
            aggregation: targetArea === 'valueFields' ? AggregationType.SUM : undefined,
        };

        // Remove from other areas if it exists (exclusive rule for rowFields/columnFields)
        if (targetArea === 'rowFields' || targetArea === 'columnFields') {
            newColumnFields = newColumnFields.filter((f) => f.sourceColumnIndex !== field.sourceColumnIndex);
            newRowFields = newRowFields.filter((f) => f.sourceColumnIndex !== field.sourceColumnIndex);
        }

        // Add to target area
        if (targetArea === 'valueFields') {
            newValueFields = [...newValueFields, newField];
        } else if (targetArea === 'rowFields') {
            newRowFields = [...newRowFields, newField];
        } else if (targetArea === 'columnFields') {
            newColumnFields = [...newColumnFields, newField];
        } else if (targetArea === 'filterFields') {
            newFilterFields = [...newFilterFields, newField];
        }

        commandService.executeCommand(UpdatePivotTableFieldsCommand.id, {
            unitId: targetCellInfo.unitId,
            subUnitId: targetCellInfo.subUnitId,
            pivotTableId: pivotTable.getId(),
            fieldsConfig: {
                valueFields: newValueFields,
                rowFields: newRowFields,
                columnFields: newColumnFields,
                filterFields: newFilterFields,
                valuePosition: pivotTable.getValuePosition(),
            } satisfies IFieldsConfig,
        } satisfies IUpdatePivotTableFieldsCommandParams);
    }, [pivotTable, commandService, valueFields, rowFields, columnFields, filterFields]);

    // Helper function to get filtered fields for dropdown
    const getFilteredFieldsForArea = useCallback((area: 'rowFields' | 'columnFields' | 'valueFields' | 'filterFields'): IPivotField[] => {
        if (area === 'valueFields') {
            // valueFields shows all fields (allowing duplicates)
            return sourceFields;
        }

        // For rowFields and columnFields, exclude fields already in rowFields or columnFields
        if (area === 'rowFields' || area === 'columnFields') {
            const exclusiveFields = new Set([
                ...rowFields.map((f) => f.sourceColumnIndex),
                ...columnFields.map((f) => f.sourceColumnIndex),
            ]);
            return sourceFields.filter((f) => !exclusiveFields.has(f.sourceColumnIndex));
        }

        // For filterFields, exclude fields already in filterFields
        if (area === 'filterFields') {
            const filterFieldIndices = new Set(filterFields.map((f) => f.sourceColumnIndex));
            return sourceFields.filter((f) => !filterFieldIndices.has(f.sourceColumnIndex));
        }

        return sourceFields;
    }, [sourceFields, rowFields, columnFields, filterFields]);

    // Memoized containers structure directly from pivotTable data
    const items: Record<string, IPivotField[]> = useMemo(() => ({
        sourceFields,
        filterFields,
        columnFields,
        rowFields,
        valueFields,
    }), [sourceFields, filterFields, columnFields, rowFields, valueFields]);

    const config = {
        sourceFields: {
            renderItem: (props) => {
                const { ref, field, ...restProps } = props;
                const isChecked = isFieldAdded(field.sourceColumnIndex);
                return (
                    <FieldRender
                        {...restProps}
                        field={field}
                        ref={ref as React.Ref<HTMLDivElement>}
                        showCheckbox={true}
                        checkboxChecked={isChecked}
                        onCheckboxChange={(checked) => {
                            if (checked) {
                                addFieldWithIntelligentPlacement(field);
                            } else {
                                removeFieldFromAllAreas(field);
                            }
                        }}
                    />
                );
            },
        },
        filterFields: {
            renderItem: (props) => {
                const { ref, field, ...restProps } = props;
                return (
                    <FieldRender
                        {...restProps}
                        field={field}
                        ref={ref as React.Ref<HTMLDivElement>}
                    // renderFooter={() => (
                    //     <div>
                    //         <div className="univer-flex univer-flex-col univer-gap-2">
                    //             <label
                    //                 className={`
                    //                   univer-text-xs univer-font-medium univer-text-gray-900
                    //                   dark:!univer-text-white
                    //                 `}
                    //             >
                    //                 状态
                    //             </label>
                    //             <SelectList
                    //                 value={['A', 'B', 'C']}
                    //                 options={[
                    //                     { label: 'A', value: 'A' },
                    //                     { label: 'B', value: 'B' },
                    //                     { label: 'C', value: 'C' },
                    //                 ]}
                    //                 onChange={() => {}}
                    //             />
                    //         </div>
                    //     </div>
                    // )}
                        onRemove={() => {
                            const targetCellInfo = pivotTable.getTargetCellInfo();
                            commandService.executeCommand(UpdatePivotTableFieldsCommand.id, {
                                unitId: targetCellInfo.unitId,
                                subUnitId: targetCellInfo.subUnitId,
                                pivotTableId: pivotTable.getId(),
                                fieldsConfig: {
                                    valueFields,
                                    rowFields,
                                    columnFields,
                                    filterFields: filterFields.filter((f) => f.id !== field.id),
                                    valuePosition: pivotTable.getValuePosition(),
                                } satisfies IFieldsConfig,
                            } satisfies IUpdatePivotTableFieldsCommandParams);
                        }}
                    />
                );
            },
        },
        columnFields: {
            renderItem: (props) => {
                const { ref, field, ...restProps } = props;
                return (
                    <FieldRender
                        {...restProps}
                        field={field}
                        ref={ref as React.Ref<HTMLDivElement>}
                        renderFooter={() => {
                            const latestRowFields = pivotTable.getRowFields();
                            const latestColumnFields = pivotTable.getColumnFields();
                            return (
                                <div className="univer-flex univer-items-center univer-gap-2">
                                    <Checkbox
                                        checked={field.showSubTotals}
                                        onChange={(checked) => {
                                            const targetCellInfo = pivotTable.getTargetCellInfo();
                                            const nextColumnFields: IPivotField[] = latestColumnFields.map((f) =>
                                                f.id === field.id ? { ...f, showSubTotals: !!checked } : f
                                            );
                                            commandService.executeCommand(UpdatePivotTableFieldsCommand.id, {
                                                unitId: targetCellInfo.unitId,
                                                subUnitId: targetCellInfo.subUnitId,
                                                pivotTableId: pivotTable.getId(),
                                                fieldsConfig: {
                                                    valueFields,
                                                    rowFields: latestRowFields,
                                                    columnFields: nextColumnFields,
                                                    filterFields,
                                                    valuePosition: pivotTable.getValuePosition(),
                                                } satisfies IFieldsConfig,
                                            } satisfies IUpdatePivotTableFieldsCommandParams);
                                        }}
                                    />
                                    <span
                                        className={`
                                          univer-text-xs univer-text-gray-900
                                          dark:!univer-text-white
                                        `}
                                    >
                                        显示总计
                                    </span>
                                </div>
                            );
                        }}
                        onRemove={() => {
                            const targetCellInfo = pivotTable.getTargetCellInfo();
                            commandService.executeCommand(UpdatePivotTableFieldsCommand.id, {
                                unitId: targetCellInfo.unitId,
                                subUnitId: targetCellInfo.subUnitId,
                                pivotTableId: pivotTable.getId(),
                                fieldsConfig: {
                                    valueFields,
                                    rowFields,
                                    columnFields: pivotTable.getColumnFields().filter((f) => f.id !== field.id),
                                    filterFields,
                                    valuePosition: pivotTable.getValuePosition(),
                                } satisfies IFieldsConfig,
                            } satisfies IUpdatePivotTableFieldsCommandParams);
                        }}
                    />
                );
            },
        },
        rowFields: {
            renderItem: (props) => {
                const { ref, field, ...restProps } = props;
                return (
                    <FieldRender
                        {...restProps}
                        field={field}
                        ref={ref as React.Ref<HTMLDivElement>}
                        renderFooter={() => {
                            const latestRowFields = pivotTable.getRowFields();
                            const latestColumnFields = pivotTable.getColumnFields();
                            return (
                                <div className="univer-flex univer-items-center univer-gap-2">
                                    <Checkbox
                                        checked={field.showSubTotals}
                                        onChange={(checked) => {
                                            const targetCellInfo = pivotTable.getTargetCellInfo();
                                            const nextRowFields: IPivotField[] = latestRowFields.map((f) =>
                                                f.id === field.id ? { ...f, showSubTotals: !!checked } : f
                                            );
                                            commandService.executeCommand(UpdatePivotTableFieldsCommand.id, {
                                                unitId: targetCellInfo.unitId,
                                                subUnitId: targetCellInfo.subUnitId,
                                                pivotTableId: pivotTable.getId(),
                                                fieldsConfig: {
                                                    valueFields,
                                                    rowFields: nextRowFields,
                                                    columnFields: latestColumnFields,
                                                    filterFields,
                                                    valuePosition: pivotTable.getValuePosition(),
                                                } satisfies IFieldsConfig,
                                            } satisfies IUpdatePivotTableFieldsCommandParams);
                                        }}
                                    />
                                    <span
                                        className={`
                                          univer-text-xs univer-text-gray-900
                                          dark:!univer-text-white
                                        `}
                                    >
                                        显示总计
                                    </span>
                                </div>
                            );
                        }}
                        onRemove={() => {
                            const targetCellInfo = pivotTable.getTargetCellInfo();
                            commandService.executeCommand(UpdatePivotTableFieldsCommand.id, {
                                unitId: targetCellInfo.unitId,
                                subUnitId: targetCellInfo.subUnitId,
                                pivotTableId: pivotTable.getId(),
                                fieldsConfig: {
                                    valueFields,
                                    rowFields: pivotTable.getRowFields().filter((f) => f.id !== field.id),
                                    columnFields,
                                    filterFields,
                                    valuePosition: pivotTable.getValuePosition(),
                                } satisfies IFieldsConfig,
                            } satisfies IUpdatePivotTableFieldsCommandParams);
                        }}
                    />
                );
            },
        },
        valueFields: {
            renderItem: (props) => {
                const { ref, field, ...restProps } = props;
                return (
                    <FieldRender
                        {...restProps}
                        field={field}
                        ref={ref as React.Ref<HTMLDivElement>}
                        renderFooter={() => {
                            const labelText = localeService.t('pivotTable.editor.aggregationMethodLabel');
                            return (
                                <div>
                                    <div className="univer-flex univer-flex-col univer-gap-2">
                                        <div
                                            className={`
                                              univer-text-xs univer-font-medium univer-text-gray-900
                                              dark:!univer-text-white
                                            `}
                                        >
                                            {labelText}
                                        </div>
                                        <Select
                                            aria-label={labelText}
                                            value={field.aggregation ?? AggregationType.SUM}
                                            options={Object.values(AggregationType).map((aggregation) => ({
                                                label: localeService.t(`pivotTable.editor.aggregationType.${aggregation}`),
                                                value: aggregation,
                                            }))}
                                            onChange={(value) => {
                                                const targetCellInfo = pivotTable.getTargetCellInfo();
                                                commandService.executeCommand(UpdatePivotTableFieldsCommand.id, {
                                                    unitId: targetCellInfo.unitId,
                                                    subUnitId: targetCellInfo.subUnitId,
                                                    pivotTableId: pivotTable.getId(),
                                                    fieldsConfig: {
                                                        valueFields: valueFields.map((f) => f.id === field.id ? { ...f, aggregation: value as AggregationType } : f),
                                                        rowFields,
                                                        columnFields,
                                                        filterFields,
                                                        valuePosition: pivotTable.getValuePosition(),
                                                    } satisfies IFieldsConfig,
                                                } satisfies IUpdatePivotTableFieldsCommandParams);
                                            }}
                                        />
                                    </div>
                                </div>
                            );
                        }}
                        onRemove={() => {
                            const targetCellInfo = pivotTable.getTargetCellInfo();
                            commandService.executeCommand(UpdatePivotTableFieldsCommand.id, {
                                unitId: targetCellInfo.unitId,
                                subUnitId: targetCellInfo.subUnitId,
                                pivotTableId: pivotTable.getId(),
                                fieldsConfig: {
                                    valueFields: valueFields.filter((f) => f.id !== field.id),
                                    rowFields,
                                    columnFields,
                                    filterFields,
                                    valuePosition: pivotTable.getValuePosition(),
                                } satisfies IFieldsConfig,
                            } satisfies IUpdatePivotTableFieldsCommandParams);
                        }}
                    />
                );
            },
        },
    } satisfies Record<string, {
        renderItem: (props: {
            field: IPivotField;
            dragOverlay: boolean;
            dragging: boolean;
            sorting: boolean;
            index: number | undefined;
            fadeIn: boolean;
            listeners: DraggableSyntheticListeners;
            ref: React.Ref<HTMLElement>;
            style: React.CSSProperties | undefined;
            transform: Transform | null;
            transition: string | null;
            value: string;
        }) => React.ReactElement;
    }>;

    const containers = useMemo(() =>
        Object.keys(items) as UniqueIdentifier[], [items]);

    const [activeId, setActiveId] = useState<UniqueIdentifier | null>(null);
    const [openDropdowns, setOpenDropdowns] = useState<Record<string, boolean>>({});
    const lastOverId = useRef<UniqueIdentifier | null>(null);
    const recentlyMovedToNewContainer = useRef(false);
    const isSortingContainer =
        activeId != null ? containers.includes(activeId) : false;

    // Toggle dropdown for a container
    const toggleDropdown = useCallback((containerId: string) => {
        setOpenDropdowns((prev) => ({
            ...prev,
            [containerId]: !prev[containerId],
        }));
    }, []);

    // Close dropdown for a container
    const closeDropdown = useCallback((containerId: string) => {
        setOpenDropdowns((prev) => ({
            ...prev,
            [containerId]: false,
        }));
    }, []);

    // Track active container for drag operations
    const activeContainerRef = useRef<UniqueIdentifier | null>(null);

    // Helper: Get base field from sourceFields by ID
    const getSourceField = useCallback((sourceColumnIndex: number): IPivotField | null => {
        // Extract base ID (remove _suffix if exists)
        return sourceFields.find((f) => f.sourceColumnIndex === sourceColumnIndex) || null;
    }, [sourceFields]);

    // Helper: Check if a field (by base ID) exists in exclusive containers
    const findFieldInExclusiveContainers = useCallback((sourceColumnIndex: number): {
        container: 'columnFields' | 'rowFields' | null;
        field: IPivotField | null;
    } => {
        // Check columnFields
        const columnField = columnFields.find((f) => f.sourceColumnIndex === sourceColumnIndex);
        if (columnField) {
            return { container: 'columnFields', field: columnField };
        }

        // Check rowFields
        const rowField = rowFields.find((f) => f.sourceColumnIndex === sourceColumnIndex);
        if (rowField) {
            return { container: 'rowFields', field: rowField };
        }

        return { container: null, field: null };
    }, [columnFields, rowFields]);

  /**
   * Custom collision detection strategy optimized for multiple containers
   *
   * - First, find any droppable containers intersecting with the pointer.
   * - If there are none, find intersecting containers with the active draggable.
   * - If there are no intersecting containers, return the last matched intersection
   *
   */
    const collisionDetectionStrategy: CollisionDetection = useCallback(
        (args) => {
            if (activeId && activeId in items) {
                return closestCenter({
                    ...args,
                    droppableContainers: args.droppableContainers.filter(
                        (container) => container.id in items
                    ),
                });
            }

            // Start by finding any intersecting droppable
            const pointerIntersections = pointerWithin(args);
            const intersections =
                pointerIntersections.length > 0
                    ? // If there are droppables intersecting with the pointer, return those
                    pointerIntersections
                    : rectIntersection(args);
            let overId = getFirstCollision(intersections, 'id');

            if (overId != null) {
                if (overId in items) {
                    const containerItems = items[overId];

                // If a container is matched and it contains items (columns 'A', 'B', 'C')
                    if (containerItems.length > 0) {
                  // Return the closest droppable within that container
                        overId = closestCenter({
                            ...args,
                            droppableContainers: args.droppableContainers.filter(
                                (container) =>
                                    container.id !== overId &&
                      containerItems.some((field) => field.id === container.id)
                            ),
                        })[0]?.id;
                    }
                }

                lastOverId.current = overId;

                return [{ id: overId }];
            }

            // When a draggable item moves to a new container, the layout may shift
            // and the `overId` may become `null`. We manually set the cached `lastOverId`
            // to the id of the draggable item that was moved to the new container, otherwise
            // the previous `overId` will be returned which can cause items to incorrectly shift positions
            if (recentlyMovedToNewContainer.current) {
                lastOverId.current = activeId;
            }

            // If no droppable is matched, return the last match
            return lastOverId.current ? [{ id: lastOverId.current }] : [];
        },
        [activeId, items]
    );
    const sensors = useSensors(
        useSensor(MouseSensor),
        useSensor(TouchSensor),
        useSensor(KeyboardSensor, {
            coordinateGetter,
        })
    );
    const findContainer = (id: UniqueIdentifier) => {
        if (id in items) {
            return id;
        }

        // Check if it's in one of the droppable containers
        return Object.keys(items).find((key) => items[key].some((field) => field.id === id));
    };

    const getIndex = (id: UniqueIdentifier) => {
        const container = findContainer(id);

        if (!container) {
            return -1;
        }

        const index = (items[container as keyof typeof items] as IPivotField[]).findIndex((field) => field.id === id);

        return index;
    };

    const onDragCancel = () => {
        setActiveId(null);
        activeContainerRef.current = null;
    };

    const handleDragEnd = ({ active, over }: DragEndEvent) => {
        const activeContainer = findContainer(active.id);

        if (!activeContainer) {
            setActiveId(null);
            return;
        }

        const overId = over?.id;

        if (overId == null) {
            setActiveId(null);
            return;
        }

        const overContainer = findContainer(overId);

        // Don't allow dropping back to source
        if (!overContainer || overContainer === 'sourceFields') {
            setActiveId(null);
            return;
        }

        const activeFieldId = String(active.id);

        const activeSourceColumnIndex = sourceFields.find((f) => f.id === activeFieldId)?.sourceColumnIndex ?? -1;

        let valueFields = pivotTable.getValueFields();
        let rowFields = pivotTable.getRowFields();
        let columnFields = pivotTable.getColumnFields();
        let filterFields = pivotTable.getFilterFields();

        // Handle dragging from source fields (copy operation)
        if (activeContainer === 'sourceFields') {
            const sourceField = getSourceField(activeSourceColumnIndex);
            if (!sourceField) {
                setActiveId(null);
                return;
            }

            const newField: IPivotField = {
                ...sourceField,
                id: `${generateRandomId(6)}_${sourceField.id}`,
            };
            // Apply Excel pivot table rules
            if (overContainer === 'valueFields') {
                // valueFields can have duplicates - create new field with unique ID
                valueFields = [...valueFields, newField];
            } else if (overContainer === 'filterFields') {
                // filterFields can only have one field per column
                const existingFilterFields = new Set(filterFields.map((f) => f.id));
                if (!existingFilterFields.has(newField.id)) {
                    filterFields = [...filterFields, newField];
                }
            } else {
                // columnFields, rowFields - exclusive rule
                const existingField = findFieldInExclusiveContainers(activeSourceColumnIndex);

                // Remove from other exclusive containers if exists
                if (existingField.container && existingField.container !== overContainer) {
                    if (existingField.container === 'columnFields') {
                        columnFields = columnFields.filter((f) => f.sourceColumnIndex !== sourceField.sourceColumnIndex);
                    } else if (existingField.container === 'rowFields') {
                        rowFields = rowFields.filter((f) => f.sourceColumnIndex !== sourceField.sourceColumnIndex);
                    }
                }

                // Add to target container
                if (overContainer === 'columnFields' && !columnFields.some((f) => f.sourceColumnIndex === sourceField.sourceColumnIndex)) {
                    columnFields = [...columnFields, existingField.field || newField];
                } else if (overContainer === 'rowFields' && !rowFields.some((f) => f.sourceColumnIndex === sourceField.sourceColumnIndex)) {
                    rowFields = [...rowFields, existingField.field || newField];
                }
            }
        } else {
            // Handle dragging between containers (not from source)
            const draggedField = [...valueFields, ...rowFields, ...columnFields, ...filterFields].find((f) => f.id === activeFieldId);

            if (!draggedField) {
                setActiveId(null);
                return;
            }

            if (activeContainer === overContainer) {
                // Reordering within same container
                const currentFields = activeContainer === 'valueFields'
                    ? valueFields :
                    activeContainer === 'rowFields'
                        ? rowFields :
                        activeContainer === 'columnFields'
                            ? columnFields :
                            filterFields;

                const activeIndex = currentFields.findIndex((f) => f.id === activeFieldId);
                const overIndex = currentFields.findIndex((f) => f.id === overId);

                if (activeIndex !== -1 && overIndex !== -1 && activeIndex !== overIndex) {
                    const reorderedFields = arrayMove(currentFields, activeIndex, overIndex);

                    if (activeContainer === 'valueFields') {
                        valueFields = reorderedFields;
                    } else if (activeContainer === 'rowFields') {
                        rowFields = reorderedFields;
                    } else if (activeContainer === 'columnFields') {
                        columnFields = reorderedFields;
                    } else if (activeContainer === 'filterFields') {
                        filterFields = reorderedFields;
                    }
                }
            } else {
                // Moving between different containers
                const sourceField = getSourceField(draggedField.sourceColumnIndex);

                if (!sourceField) {
                    setActiveId(null);
                    return;
                }

                // Remove from source container first
                if (activeContainer === 'valueFields') {
                    valueFields = valueFields.filter((f) => f.id !== activeFieldId);
                } else if (activeContainer === 'rowFields') {
                    rowFields = rowFields.filter((f) => f.id !== activeFieldId);
                } else if (activeContainer === 'columnFields') {
                    columnFields = columnFields.filter((f) => f.id !== activeFieldId);
                } else if (activeContainer === 'filterFields') {
                    filterFields = filterFields.filter((f) => f.id !== activeFieldId);
                }

                const newField: IPivotField = {
                    ...sourceField,
                    id: `${generateRandomId(6)}_${sourceField.id}`,
                };

                // Add to target container
                if (overContainer === 'valueFields') {
                    // Moving to valueFields - create new field with unique ID
                    valueFields = [...valueFields, newField];
                } else if (overContainer === 'filterFields') {
                    // filterFields can only have one field per column
                    const existingFilterFields = new Set(filterFields.map((f) => f.id));
                    if (!existingFilterFields.has(newField.id)) {
                        filterFields = [...filterFields, newField];
                    }
                } else {
                    // Moving to exclusive container - remove from others first
                    if (overContainer === 'rowFields') {
                        columnFields = columnFields.filter((f) => f.sourceColumnIndex !== sourceField.sourceColumnIndex);
                    } else if (overContainer === 'columnFields') {
                        rowFields = rowFields.filter((f) => f.sourceColumnIndex !== sourceField.sourceColumnIndex);
                    }

                    // Add to target
                    if (overContainer === 'columnFields') {
                        columnFields = [...columnFields, newField];
                    } else if (overContainer === 'rowFields') {
                        rowFields = [...rowFields, newField];
                    }
                }
            }
        }

        const targetCellInfo = pivotTable.getTargetCellInfo();

        // Update user preferences when fields are moved between areas
        if (activeContainer !== 'sourceFields' && overContainer && overContainer !== 'sourceFields') {
            const movedField = [...valueFields, ...rowFields, ...columnFields, ...filterFields].find(
                (f) => f.sourceColumnIndex === activeSourceColumnIndex
            );
            if (movedField) {
                const sourceField = getSourceField(activeSourceColumnIndex);
                if (sourceField) {
                    intelligentPlacementService.updatePreference({
                        valueFields,
                        rowFields,
                        columnFields,
                        filterFields,
                        valuePosition: pivotTable.getValuePosition(),
                    });
                }
            }
        }

        commandService.executeCommand(UpdatePivotTableFieldsCommand.id, {
            unitId: targetCellInfo.unitId,
            subUnitId: targetCellInfo.subUnitId,
            pivotTableId: pivotTable.getId(),
            fieldsConfig: {
                valueFields,
                rowFields,
                columnFields,
                filterFields,
                valuePosition: pivotTable.getValuePosition(),
            },
        } satisfies IUpdatePivotTableFieldsCommandParams);

        setActiveId(null);
        activeContainerRef.current = null;
    };

    return (
        <DndContext
            sensors={sensors}
            collisionDetection={collisionDetectionStrategy}
            measuring={{
                droppable: {
                    strategy: MeasuringStrategy.Always,
                },
            }}
            onDragStart={(params) => {
                const { active } = params;
                const container = findContainer(active.id);
                setActiveId(active.id);
                activeContainerRef.current = container || null;
            }}
            onDragOver={({ over }) => {
                // Track the current over target for visual feedback
                if (over?.id) {
                    lastOverId.current = over.id;
                }
            }}
            onDragEnd={handleDragEnd}
            cancelDrop={cancelDrop}
            onDragCancel={onDragCancel}
            modifiers={modifiers}
        >
            <div className="univer-grid univer-grid-cols-2 univer-gap-2">
                {containers.map((containerId) => {
                    // Don't show Add button for sourceFields
                    const showAddButton = containerId !== 'sourceFields';
                    const filteredFields = showAddButton ? getFilteredFieldsForArea(containerId as 'rowFields' | 'columnFields' | 'valueFields' | 'filterFields') : [];
                    const isDropdownOpen = openDropdowns[String(containerId)] ?? false;

                    return (
                        <DroppableContainer
                            className={containerId === 'rowFields' || containerId === 'columnFields'
                                ? 'univer-col-span-1'
                                : 'univer-col-span-2'}
                            key={containerId}
                            id={containerId}
                            label={minimal ? undefined : localeService.t(`pivotTable.editor.containerLabels.${String(containerId)}`)}
                            columns={columns}
                            items={items[containerId].map((field) => field.id)}
                            scrollable={scrollable}
                            style={containerStyle}
                            unstyled={minimal}
                            showAddButton={showAddButton}
                            addButtonLabel={localeService.t('pivotTable.editor.add')}
                            addButtonOverlay={showAddButton && filteredFields.length > 0
                                ? (
                                    <Dropdown
                                        open={isDropdownOpen}
                                        onOpenChange={(open) => {
                                            if (open) {
                                                toggleDropdown(String(containerId));
                                            } else {
                                                closeDropdown(String(containerId));
                                            }
                                        }}
                                        overlay={(
                                            <SelectList
                                                value=""
                                                options={filteredFields.map((field) => ({
                                                    label: field.name,
                                                    value: String(field.sourceColumnIndex),
                                                }))}
                                                onChange={(value) => {
                                                    if (value) {
                                                        const fieldIndex = Number.parseInt(Array.isArray(value) ? value[0] : value, 10);
                                                        const field = sourceFields.find((f) => f.sourceColumnIndex === fieldIndex);
                                                        if (field) {
                                                            addFieldToArea(field, containerId as 'rowFields' | 'columnFields' | 'valueFields' | 'filterFields');
                                                            closeDropdown(String(containerId));
                                                        }
                                                    }
                                                }}
                                            />
                                        )}
                                    >
                                        <Button size="small">{localeService.t('pivotTable.editor.add')}</Button>
                                    </Dropdown>
                                )
                                : undefined}
                        >
                            <SortableContext items={items[containerId].map((field) => field.id)} strategy={strategy}>
                                {items[containerId]?.map((field: IPivotField, index: number) => {
                                    return (
                                        <SortableItem
                                            disabled={isSortingContainer}
                                            key={field.id}
                                            id={field.id}
                                            index={index}
                                            handle={handle}
                                            style={getItemStyles}
                                            wrapperStyle={wrapperStyle}
                                            renderItem={(props) => config[containerId as keyof typeof config].renderItem({
                                                ...props,
                                                field,
                                            })}
                                            containerId={containerId}
                                            getIndex={getIndex}
                                            value={field.name}
                                        />
                                    );
                                })}
                            </SortableContext>
                        </DroppableContainer>
                    );
                })}
            </div>
            {createPortal(
                <DragOverlay adjustScale={adjustScale} dropAnimation={dropAnimation}>
                    {activeId
                        ? containers.includes(activeId)
                            ? renderContainerDragOverlay(activeId)
                            : renderSortableItemDragOverlay(activeId)
                        : null}
                </DragOverlay>,
                document.body
            )}
        </DndContext>
    );

    function renderSortableItemDragOverlay(id: UniqueIdentifier) {
        // Find the field object to get its name
        const field = Object.values(items)
            .flat()
            .find((f) => f.id === id);
        const fieldName = field?.name ?? String(id);

        return (
            <FieldItem
                id={String(id)}
                value={fieldName}
                handle={handle}
                style={getItemStyles({
                    containerId: findContainer(id) as UniqueIdentifier,
                    overIndex: -1,
                    index: getIndex(id),
                    value: id,
                    isSorting: true,
                    isDragging: true,
                    isDragOverlay: true,
                })}
                wrapperStyle={wrapperStyle({ index: 0 })}
                renderItem={renderItem}
                dragOverlay
            />
        );
    }

    function renderContainerDragOverlay(containerId: UniqueIdentifier) {
        return (
            <FieldItemsContainer
                label={localeService.t('pivotTable.editor.containerLabel', String(containerId))}
                columns={columns}
                style={{
                    height: '100%',
                }}
                shadow
                unstyled={false}
            >
                {items[String(containerId)].map((item: IPivotField, index: number) => (
                    <FieldItem
                        id={item.id}
                        key={item.id}
                        value={item.name}
                        handle={handle}
                        style={getItemStyles({
                            containerId,
                            overIndex: -1,
                            index: getIndex(item.id),
                            value: item.id,
                            isDragging: false,
                            isSorting: false,
                            isDragOverlay: false,
                        })}
                        wrapperStyle={wrapperStyle({ index })}
                        renderItem={renderItem}
                    />
                ))}
            </FieldItemsContainer>
        );
    }
}

interface ISortableItemProps {
    containerId: UniqueIdentifier;
    id: UniqueIdentifier;
    index: number;
    handle: boolean;
    disabled?: boolean;
    value: string;
    style(args: {
        value: UniqueIdentifier;
        index: number;
        overIndex: number;
        isDragging: boolean;
        containerId: UniqueIdentifier;
        isSorting: boolean;
    }): React.CSSProperties;
    getIndex(id: UniqueIdentifier): number;
    renderItem?(props: {
        dragOverlay: boolean;
        dragging: boolean;
        sorting: boolean;
        index: number | undefined;
        fadeIn: boolean;
        listeners: DraggableSyntheticListeners;
        ref: React.Ref<HTMLElement>;
        style: React.CSSProperties | undefined;
        transform: Transform | null;
        transition: string | null;
        value: string;
    }): React.ReactElement;
    wrapperStyle({ index }: { index: number }): React.CSSProperties;
}

function SortableItem({
    disabled,
    id,
    index,
    handle,
    renderItem,
    style,
    containerId,
    getIndex,
    wrapperStyle,
    value,
}: ISortableItemProps) {
    const {
        setNodeRef,
        setActivatorNodeRef,
        listeners,
        isDragging,
        isSorting,
        over,
        overIndex,
        transform,
        transition,
    } = useSortable({
        id,
    });
    const mounted = useMountStatus();
    const mountedWhileDragging = isDragging && !mounted;

    return (
        <FieldItem
            ref={disabled ? undefined : setNodeRef}
            id={String(id)}
            value={value}
            dragging={isDragging}
            sorting={isSorting}
            handle={handle}
            handleProps={handle ? { ref: setActivatorNodeRef } : undefined}
            index={index}
            wrapperStyle={wrapperStyle({ index })}
            style={style({
                index,
                value: id,
                isDragging,
                isSorting,
                overIndex: over ? getIndex(over.id) : overIndex,
                containerId,
            })}
            transition={transition}
            transform={transform}
            fadeIn={mountedWhileDragging}
            listeners={listeners}
            renderItem={renderItem}
        />
    );
}

function useMountStatus() {
    const [isMounted, setIsMounted] = useState(false);

    useEffect(() => {
        const timeout = setTimeout(() => setIsMounted(true), 500);

        return () => clearTimeout(timeout);
    }, []);

    return isMounted;
}
