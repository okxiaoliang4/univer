# Project Context

## Purpose

Univer is an **isomorphic full-stack framework** for creating and editing **spreadsheets**, **documents**, and **presentations** across web and server environments. The project aims to provide:

- **Extensible**: Plugin architecture allowing custom functionality
- **High-performance**: Canvas-based rendering engine, lightning-fast formula engine in Web Workers
- **Embeddable**: Easy integration into any application
- **Feature-rich**: Formulas, conditional formatting, data validation, collaborative editing, import/export
- **Isomorphic**: Runs identically in browsers and Node.js (future: mobile)
- **Highly integrated**: Documents, sheets, and slides can interoperate on the same canvas

**License**: Apache-2.0 | **Homepage**: https://univer.ai | **Repo**: https://github.com/dream-num/univer

## Tech Stack

### Core Technologies
- **TypeScript 5.9+**: Primary language across all packages
- **React 19**: UI framework (supports React 16-19 via version manager)
- **Node.js >= 20**: Runtime requirement
- **pnpm >= 10**: Package manager

### Build & Development
- **Turbo**: Monorepo task orchestration
- **Vite 7**: Build tool and dev server
- **ESBuild**: High-speed bundling
- **tsx**: TypeScript execution

### Rendering & Computation
- **Canvas API**: Custom rendering engine with advanced typesetting (punctuation squeezing, text-image layout, scroll buffering)
- **Web Workers**: Formula engine execution for performance
- **RxJS 7+**: Reactive state management and data flow

### Testing & Quality
- **Vitest**: Unit testing framework
- **Playwright**: E2E and visual regression testing
- **Storybook**: Component development and testing in isolation

### Styling
- **Tailwind CSS 3.4**: Utility-first CSS framework
- **PostCSS**: CSS processing

### Tooling
- **ESLint**: Linting (using @antfu/eslint-config)
- **Husky**: Git hooks
- **commitlint**: Commit message linting (conventional commits)
- **lint-staged**: Pre-commit linting

## Project Conventions

### Code Style

#### Naming Conventions
- **Files & Folders**: Use `kebab-case` for file/folder names, except React components which use `PascalCase`
  - ✅ `my-component/MyComponent.tsx`
  - ✅ `services/log.service.ts`
  - ❌ `myComponent/my-component.tsx`

- **Folders**: Plural format (e.g., `services/`, `controllers/`, `commands/`)
- **Files**: Singular format (e.g., `service.ts`, `controller.ts`)

- **Type suffixes**: Use conventional suffixes: `.service`, `.controller`, `.command`, `.mutation`, `.operation`

- **Interfaces**: Prefix with capital `I`
  - ✅ `export interface IMyInterface {}`
  - ❌ `export interface MyInterface {}`

- **Dependency Injection Tokens**: Format `<package-name>.<name>.<type>`
  - ✅ `createIdentifier<ILogService>('core.log.service')`

- **Plugin Names**: `<BUSINESS_TYPE>_<PLUGIN_NAME>_PLUGIN` in SCREAMING_SNAKE_CASE
  - ✅ `SHEET_CONDITIONAL_FORMATTING_PLUGIN`

- **Plugin Classes**: PascalCase prefixed with `Univer`
  - ✅ `export class UniverFilterPlugin extends Plugin {}`

- **Commands**: Format `<business-type>.<command-type>.<command-name>` (singular business-type)
  - ✅ `id: 'sheet.command.set-selection-frozen'`
  - ❌ `id: 'sheets.command.set-selection-frozen'`

- **IDs**: Always use lowercase: `id` (not `Id`)

#### Formatting
- Managed by ESLint with @antfu/eslint-config
- Auto-fixed via lint-staged on commit
- No exposed properties/methods unless necessary
- Group related methods/properties together
- Consistent naming across similar concepts

### Architecture Patterns

#### Monorepo Structure
```
packages/          # Core and published plugins
packages-experimental/  # Experimental, unpublished plugins
common/            # Shared configuration and utilities
examples/          # Web demos
e2e/               # End-to-end tests
docs/              # Documentation
```

#### Plugin Architecture
All plugins follow a strict folder structure:
```
common/            # Cannot import from other folders
models/            # Can only import from common/
services/          # Can only import from models/, common/
commands/          # Can only import from services/, models/, common/
  commands/
  mutations/
  operations/
controllers/       # Business logic coordination
views/             # UI components
  components/
  parts/
plugin.ts          # Plugin entry point
index.ts           # Public API exports
```

**Import Restrictions**:
- `common/` → Cannot import from other folders
- `models/` → Only `common/`
- `services/` → Only `models/`, `common/`
- `commands/` → Only `services/`, `models/`, `common/`

**Platform-specific UI** (Desktop & Mobile support since June 2024):
```
controllers/render-controllers/
  common/
  desktop/
  mobile/
views/components/
  common/
  desktop/
  mobile/
```

#### Dependency Injection
- Uses custom DI system
- Services and controllers registered via `createIdentifier`
- Resource keys must match plugin names

#### Isomorphic Design
- Same API works in browser and Node.js
- Formula engine can run in Web Workers or server-side
- Rendering engine supports both environments

#### No Barrel Imports
- Avoid creating `index.ts` files for re-exports
- Exception: Main plugin entry point

### Testing Strategy

#### Unit Tests
- **Framework**: Vitest
- **Requirement**: All code must be covered by unit tests
- **Location**: Co-located with source files (typically `__tests__/` folders)
- **Command**: `pnpm test` (or `pnpm test:watch` for watch mode)
- **Coverage**: `pnpm coverage`
- **PR Requirement**: Test coverage must not decrease

#### E2E Tests
- **Framework**: Playwright
- **Types**: Smoke tests, visual regression, memory leak detection, performance
- **Command**: `pnpm test:e2e`
- **Dev server**: `pnpm dev:e2e`
- **Installation**: May need `pnpm exec playwright install`

#### Visual Regression
- Playwright snapshots for UI changes
- Auto-updated via GitHub Action on PRs
- Located in `e2e/visual-comparison/`

#### Component Testing
- **Storybook**: Component development and testing in isolation
- **Commands**: `pnpm storybook:dev`, `pnpm storybook:build`
- Auto-deployed on PRs

#### Test Organization
- Each package has its own `vitest.config.ts`
- Workspace configured in `vitest.workspace.ts`
- Tests must pass before merging PRs

### Git Workflow

#### Commit Conventions
- **Format**: Conventional Commits (enforced by commitlint)
- **Config**: `@commitlint/config-conventional`
- **Examples**:
  - `feat: add two-factor authentication`
  - `fix: resolve formula calculation error`
  - `docs: update API reference`
  - `refactor: simplify render pipeline`

#### Branching Strategy
- `dev`: Main development branch
- Feature branches: Created from `dev`
- PRs: Merged into `dev` after review

#### Pre-commit Hooks
- **Husky**: Manages git hooks
- **lint-staged**: Runs `eslint --fix` on staged files
- All linting and formatting errors must be fixed before commit

#### PR Requirements
- ✅ All tests pass
- ✅ ESLint and Prettier errors fixed
- ✅ Test coverage maintained or improved
- ✅ Preview deployments available for review
- ✅ Visual regression tests pass (or snapshots updated)

#### Context Connection
- Include issue links in commit messages
- Add code comments with relevant context links
- Document important information in codebase

## Domain Context

### Office Suite Functionality
Univer provides comprehensive office suite capabilities:

#### Spreadsheets (Sheets)
- **Core**: Cells, rows, columns, worksheets, workbooks
- **Formulas**: 200+ functions (math, statistical, logical, text, date/time, lookup, financial, engineering)
- **Features**: Number formatting, hyperlinks, floating images, find & replace, filtering, sorting, data validation, conditional formatting, comments, cross-highlighting, Zen editor
- **Advanced**: Pivot tables, sparklines, charts, printing, XLSX import/export, collaborative editing, history

#### Documents (Docs) - RC Stage
- **Core**: Paragraphs, headings, lists, superscript, subscript
- **Features**: Lists (ordered, unordered, task), hyperlinks, floating images with text layout, headers & footers, comments
- **Advanced**: Printing, DOCX import/export, collaborative editing

#### Presentations (Slides) - Under Development
- **Core**: Slides, shapes, text, images

### Internationalization
- **Supported**: zh-CN, zh-TW, en-US, ru-RU, vi-VN, fa-IR, ko-KR, es-ES, ca-ES
- **Official**: zh-CN, en-US (others community-maintained)
- Custom locales supported

### Performance Characteristics
- Canvas-based rendering for large datasets
- Formula engine in Web Workers for non-blocking computation
- Scroll buffering for smooth large-document rendering
- Advanced typesetting: punctuation squeezing, complex text-image layouts

## Important Constraints

### Technical Constraints
1. **Isomorphic Requirement**: All core functionality must work identically in browser and Node.js
2. **Plugin Architecture**: Features must be implemented as plugins following the strict folder structure
3. **No Breaking Changes**: API deprecation follows a careful process (JSDoc + ILogService warnings, removal in next minor/major)
4. **Platform Support**: UI plugins must support both desktop and mobile since June 2024
5. **Node Version**: Requires Node.js >= 20
6. **Package Manager**: Must use pnpm >= 10 (not npm or yarn)
7. **Import Restrictions**: Strict enforcement of folder-level import rules (common → models → services → commands)

### Build Constraints
- **Turbo**: All builds coordinated through Turbo with concurrency limits
- **Filter**: Production builds exclude `./common/*` packages
- **Dual Output**: Packages must provide both ESM and CJS outputs
- **Types**: TypeScript declaration files required for all public APIs

### Performance Constraints
- Canvas rendering must maintain 60fps for typical operations
- Formula calculations should not block UI thread (use Web Workers)
- Initial bundle size must be optimized (tree-shakeable exports)

### Licensing
- **License**: Apache-2.0 for OSS version
- **Non-OSS Features**: Some advanced features (collaborative editing, pivot tables, import/export) available in free-for-commercial non-OSS version

## External Dependencies

### Key Runtime Dependencies
- **RxJS 7.x**: Reactive state management (peer dependency across packages)
- **@flatten-js/interval-tree**: Data structure for formula engine
- **decimal.js**: Precise decimal arithmetic in formula engine

### Rendering Stack
- **Canvas API**: Core rendering (no external canvas libraries)
- Custom rendering engine in `@univerjs/engine-render`

### Formula Engine
- **@univerjs/engine-formula**: Custom formula engine
- **@univerjs/engine-numfmt**: Number formatting
- Operates in Web Workers via `@univerjs/rpc`

### UI Framework
- **React 19** (with backwards compatibility to React 16)
- **Tailwind CSS**: Utility-first styling
- **@univerjs/design**: Design system components

### Development Dependencies
- **Vite**: Development server and building
- **Turbo**: Monorepo orchestration
- **Vitest**: Testing framework
- **Playwright**: E2E testing
- **ESLint**: Code linting
- **TypeScript**: Type checking

### Build & Publishing
- **release-it**: Automated releases
- **@release-it-plugins/workspaces**: Monorepo release support
- **@release-it/conventional-changelog**: Changelog generation

### Services & Infrastructure
- **GitHub Actions**: CI/CD pipelines
- **Vercel**: Preview deployments
- **Open Collective**: Funding platform
- **Discord**: Community support
- **codecov**: Code coverage reporting

### Optional Integrations
- **Univer MCP**: Natural language interface for spreadsheets
- **PostHog**: Analytics (dev dependency)
- **Vue 3**: Alternative UI adapter via `@univerjs/ui-adapter-vue3`
- **Web Components**: Alternative UI adapter via `@univerjs/ui-adapter-web-component`
