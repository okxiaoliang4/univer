# @univerjs/sheets-pivot-table

## Introduction

> Core functionality for pivot tables in Univer Sheet.

This plugin provides the core functionality for creating, managing, and calculating pivot tables in Univer spreadsheets. It enables users to aggregate and analyze large datasets through dynamic row and column groupings with support for multiple aggregation functions.

## Features

- ✅ Create pivot tables from worksheet data ranges
- ✅ Configure row fields, column fields, value fields, and filter fields
- ✅ Support for 5 aggregation functions: SUM, COUNT, AVERAGE, MIN, MAX
- ✅ Snapshot serialization/deserialization for save/load
- ✅ Service-based architecture for extensibility
- ✅ Command/Mutation pattern for undo/redo support

### Installation

```shell
npm i @univerjs/sheets-pivot-table
```

### Import

```ts
import { UniverSheetsPivotTablePlugin } from '@univerjs/sheets-pivot-table';

univer.registerPlugin(UniverSheetsPivotTablePlugin);
```

## License

Apache-2.0

<!-- Links -->
[npm-version-shield]: https://img.shields.io/npm/v/@univerjs/sheets-pivot-table?style=flat-square
[npm-version-link]: https://npmjs.com/package/@univerjs/sheets-pivot-table
[npm-license-shield]: https://img.shields.io/npm/l/@univerjs/sheets-pivot-table?style=flat-square
[npm-downloads-shield]: https://img.shields.io/npm/dm/@univerjs/sheets-pivot-table?style=flat-square
