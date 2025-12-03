## ADDED Requirements

### Requirement: Pivot Table Formula Integration

The system SHALL enable formulas to reference pivot table output cells and calculate correctly, without writing pivot output values to the worksheet model.

This is implemented using the Formula Engine's **Feature Calculation** mechanism:
- **Feature Registration**: Pivot table registers as a feature in `IFeatureCalculationManagerService`
- **Runtime Data Provider**: Feature's `getDirtyData` callback provides pivot output values to formula engine
- **Dirty Range Propagation**: Formula engine marks dependents dirty when pivot data changes

#### Scenario: Formula references single pivot output cell
- **WHEN** a formula in cell E1 contains `=C3`
- **AND** cell C3 is within a pivot table output range
- **AND** the pivot table has calculated value 100 for that cell
- **THEN** the `PivotTableFormulaController` registers the pivot table as a feature
- **AND** the feature's `getDirtyData` callback returns the pivot output matrix
- **AND** the formula engine's `getRuntimeFeatureCellValue()` retrieves value 100
- **AND** cell E1 displays 100
- **AND** the worksheet model's cell C3 remains empty (no model pollution)

#### Scenario: Formula with operations on pivot output cells
- **WHEN** a formula in cell E1 contains `=C3+C4`
- **AND** cells C3 and C4 are within pivot table output range
- **AND** the pivot table has calculated values 100 and 200 for those cells
- **THEN** the formula engine retrieves both values from `getRuntimeFeatureCellValue()`
- **AND** cell E1 displays 300 (100 + 200)

#### Scenario: Formula references pivot output from different worksheet
- **WHEN** a formula in Sheet2!A1 contains `=Sheet1!C3`
- **AND** Sheet1!C3 is within a pivot table output range on Sheet1
- **THEN** the formula engine correctly resolves the cross-sheet reference
- **AND** retrieves the pivot value from the feature's runtime data
- **AND** displays the correct value in Sheet2!A1

#### Scenario: Formula recalculates when pivot source data changes
- **WHEN** a formula in E1 references pivot output cell C3 (`=C3`)
- **AND** the pivot table source data changes (e.g., value in A1 is updated)
- **THEN** the pivot table recalculates its output
- **AND** the feature's `getDirtyData` returns updated values
- **AND** the formula engine marks E1 as dirty
- **AND** E1 recalculates with the new pivot value

#### Scenario: Feature registration on pivot table creation
- **WHEN** a new pivot table is created via `SheetsPivotTableService.createPivotTable()`
- **THEN** the `PivotTableFormulaController` listens to `pivotTableAdded$`
- **AND** executes `SetFeatureCalculationMutation` with the pivot's feature ID
- **AND** provides `dependencyRanges` covering the pivot output range
- **AND** the feature is registered in `IFeatureCalculationManagerService`

#### Scenario: Feature unregistration on pivot table deletion
- **WHEN** a pivot table is deleted via `deletePivotTable()`
- **THEN** the `PivotTableFormulaController` executes `RemoveFeatureCalculationMutation`
- **AND** the feature is removed from `IFeatureCalculationManagerService`
- **AND** formulas that referenced the pivot output now see empty cells (or #REF! if appropriate)

#### Scenario: Feature update when pivot output range changes
- **WHEN** a pivot table's target cell is moved via `updateTargetCell()`
- **OR** the pivot output dimensions change due to field configuration
- **THEN** the `PivotTableFormulaController` updates the feature's `dependencyRanges`
- **AND** formulas referencing old positions no longer find pivot data
- **AND** formulas referencing new positions find correct pivot data

#### Scenario: Multiple pivot tables in same worksheet
- **WHEN** a worksheet contains multiple pivot tables
- **AND** formulas reference cells from different pivot tables
- **THEN** each pivot table is registered as a separate feature
- **AND** `getRuntimeFeatureCellValue()` correctly identifies which feature provides each cell's value
- **AND** all formulas calculate correctly

#### Scenario: Performance with large pivot output
- **WHEN** a pivot table has 1000+ output cells
- **AND** multiple formulas reference these cells
- **THEN** the `getDirtyData` callback returns pre-calculated data from `getOutputCellMatrix()`
- **AND** no expensive recalculation occurs in the callback
- **AND** formula calculation time remains acceptable (no significant degradation)

