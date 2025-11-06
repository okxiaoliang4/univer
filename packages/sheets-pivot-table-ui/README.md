# @univerjs/sheets-pivot-table-ui

## Introduction

> UI layer for pivot tables in Univer Sheet.

This package provides the user interface components and interactions for pivot tables in Univer spreadsheets. It depends on `@univerjs/sheets-pivot-table` for core functionality.

## Features (MVP)

- ✅ Plugin registration and lifecycle management
- ✅ Service layer for UI state management
- ✅ Menu and shortcut integration
- ✅ Operations for UI interactions
- 🚧 React UI components (coming soon)
- 🚧 Drag-and-drop field configuration (coming soon)
- 🚧 Pivot table rendering (coming soon)

### Installation

```shell
npm i @univerjs/sheets-pivot-table-ui
```

### Import

```ts
import { UniverSheetsPivotTablePlugin } from '@univerjs/sheets-pivot-table';
import { UniverSheetsPivotTableUIPlugin } from '@univerjs/sheets-pivot-table-ui';

// Register core plugin first
univer.registerPlugin(UniverSheetsPivotTablePlugin);

// Then register UI plugin
univer.registerPlugin(UniverSheetsPivotTableUIPlugin);
```

## License

Apache-2.0

<!-- Links -->
[npm-version-shield]: https://img.shields.io/npm/v/@univerjs/sheets-pivot-table-ui?style=flat-square
[npm-version-link]: https://npmjs.com/package/@univerjs/sheets-pivot-table-ui
[npm-license-shield]: https://img.shields.io/npm/l/@univerjs/sheets-pivot-table-ui?style=flat-square
[npm-downloads-shield]: https://img.shields.io/npm/dm/@univerjs/sheets-pivot-table-ui?style=flat-square
