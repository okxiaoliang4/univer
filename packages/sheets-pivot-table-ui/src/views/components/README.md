# Pivot Table UI Components

## PivotTableEditor

A modern, drag-and-drop interface for configuring pivot table fields, similar to Excel's pivot table field panel but with a more contemporary design.

### Features

- **Drag and Drop**: Easily drag fields from the available fields list to different areas (Filters, Columns, Rows, Values)
- **Visual Feedback**: Real-time visual feedback during drag operations
- **Field Management**:
  - View all available fields from the source data
  - Drag fields between different areas
  - Remove fields by clicking the × button
  - Disabled state for fields already in use
- **Modern UI**: Clean, accessible interface with proper ARIA attributes
- **Responsive Layout**: Grid-based layout for the four drop zones
- **Aggregation Display**: Shows aggregation type (SUM) for value fields

### Usage

The `PivotTableEditor` is used within the `PivotTablePanel` component:

```tsx
import { PivotTableEditor } from './PivotTableEditor';

<PivotTableEditor pivotTable={pivotTable} />
```

### Props

| Prop | Type | Description |
|------|------|-------------|
| `pivotTable` | `PivotTable` | The pivot table instance to configure |

### Structure

The editor is divided into several sections:

1. **Header**: Displays the panel title
2. **Available Fields**: Shows all fields that can be dragged to the drop zones
3. **Drop Zones** (2x2 grid):
   - **Filters**: Fields used to filter the data
   - **Columns**: Fields displayed as column headers
   - **Rows**: Fields displayed as row headers
   - **Values**: Fields to aggregate and display as values

### Localization

All text is localized using the `LocaleService`. Required locale keys:

- `pivotTable.editor.title`
- `pivotTable.editor.availableFields`
- `pivotTable.editor.filters`
- `pivotTable.editor.columns`
- `pivotTable.editor.rows`
- `pivotTable.editor.values`
- `pivotTable.editor.dragFieldsHere`
- `pivotTable.editor.remove`

### Styling

The component uses CSS Modules for styling (`PivotTableEditor.module.css`). Key features:

- Modern color scheme with subtle shadows and transitions
- Hover effects on draggable elements
- Drag-over state highlighting
- Dark mode support
- Scrollable field lists with custom scrollbars

### Accessibility

- Proper ARIA labels and roles
- Keyboard accessible buttons
- Screen reader friendly
- Semantic HTML structure

