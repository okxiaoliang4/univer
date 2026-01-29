
# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Univer is an isomorphic full-stack framework for creating and editing spreadsheets, documents, and presentations across web and server environments. It uses a plugin-based architecture with a canvas-based rendering engine and a formula calculation engine that can run in Web Workers or server-side.

**Tech Stack:**
- Runtime: Node.js >= 20.0.0, pnpm >= 10.25.0
- Module System: ES modules (type: "module")
- Framework: React 19.2.3
- Build: Turbo (monorepo orchestration), ESBuild (examples)
- Testing: Vitest (unit), Playwright (E2E)
- Language: TypeScript 5.9.3

## Monorepo Structure

```
/Volumes/data/projects/univer/
├── packages/                  # Core OSS packages
├── packages-experimental/    # Experimental plugins (not published to npm)
├── examples/                 # Demo applications
├── common/                   # Shared configuration
├── tests/                    # Test utilities
├── mockdata/                 # Mock data for demos
├── docs/                     # Documentation
├── e2e/                      # End-to-end tests
└── scripts/                  # Build and utility scripts
```

## Common Development Commands

```bash
# Install dependencies
pnpm install

# Start demo development server
pnpm dev

# Build all packages
pnpm build

# Build for CI (100% concurrency)
pnpm build:ci

# Build examples/demo
pnpm build:demo

# Type checking
pnpm typecheck

# Run all tests
pnpm test

# Run tests with coverage
pnpm coverage

# Run E2E tests
pnpm test:e2e

# Lint code
pnpm lint

# Start Storybook
pnpm storybook:dev
```

**Running Single Tests:**
- Unit tests use Vitest with workspace support
- Each package has its own `test` script in package.json
- Use `pnpm --filter <package-name> test` to run tests for a specific package

## Architecture Principles

### Plugin Architecture

Univer uses a highly modular plugin system. Each feature is typically split into:

1. **Core Logic Plugin**: Contains models, commands, mutations, services - runs on both browser and Node.js
2. **UI Plugin** (suffixed with `-ui`): Contains React components, menus, canvas elements - browser only

Example: `sheets-filter` (logic) + `sheets-filter-ui` (UI)

### Isomorphic Design

The framework is designed to run identically on browsers and Node.js:

- Separate logic from UI to enable server-side execution
- Commands and mutations should not read UI state directly
- Facade API should be implemented in core logic plugins
- Server-only features (using `fs`, `path`, etc.) go in separate plugins

### Dependency Injection

Uses `@wendellhu/redi` for dependency injection. Services and controllers are injected via the DI container with identifier tokens.

**DI Token Naming:**
```typescript
export const IYourServiceName = createIdentifier<IYourServiceName>('<package-name>.<service-name>.service');
```

### Package Organization

Each plugin follows this structure:
```
|- common/           # Shared utilities (cannot import from other folders)
|- models/           # Data models (can only import from common)
|- services/         # Business logic (can import from models, common)
|- commands/         # Command definitions
  |- commands/
  |- mutations/
  |- operations/
|- controllers/      # Control logic
|- views/            # UI components
  |- components/
  |- parts/
|- plugin.ts         # Plugin entry point
|- index.ts          # Package exports
```

**Import Restrictions:**
- common cannot import from other folders in the same plugin
- models can only import from common
- services can only import from models and common
- commands can import from common, models, and services

**Desktop & Mobile Separation:**
UI code should be split by platform:
```
|- controllers/
  |- render-controllers/
    |- common/
    |- desktop/
    |- mobile/
|- views/
  |- components/
    |- common/
    |- desktop/
    |- mobile/
```

## Naming Conventions

**Files & Folders:**
- Use kebab-case for files and folders
- React components use PascalCase: `MyComponent.tsx`
- Folders are plural: `services/`, `components/`
- Files are singular: `user.service.ts`
- Use suffixes for type: `.service.ts`, `.controller.ts`, `.command.ts`, `.mutation.ts`

**Interfaces:**
- Prefix with capital `I`: `IMyInterface`

**Plugin Names:**
- Constants: `<BUSINESS_TYPE>_<PLUGIN_NAME>_PLUGIN`
- Classes: `Univer` prefix, PascalCase: `UniverFilterPlugin`

**Commands:**
```typescript
// Command ID format: '<business-type>.<command-type>.<command-name>'
export const SomeCommand: ICommand<ISomeCommandParams> = {
    id: 'sheet.command.set-selection-frozen',
};
```

**IDs:** Always use `id` or `Id` (pascal case)

## Facade API Guidelines

The Facade API provides a simplified user-facing API layer. When contributing:

- Reference Google AppsScript for API design patterns
- Use JSDoc for all public APIs
- Prefer synchronous APIs; suffix async APIs with `Async`
- Follow chaining principle:
  - `modify` semantics return `this`
  - `create` semantics return the created instance
  - `delete` semantics return `boolean`
- All APIs/constants/enums should be accessible from the `univerAPI` variable

## Key Packages

**Core Framework:**
- `@univerjs/core` - Core framework, document model, undo/redo, plugin system
- `@univerjs/engine-render` - Canvas-based rendering engine
- `@univerjs/engine-formula` - Formula calculation engine
- `@univerjs/rpc` - Remote procedure call system

**Document Types:**
- `@univerjs/sheets` - Spreadsheet functionality
- `@univerjs/docs` - Document/rich text processing
- `@univerjs/slides` - Presentations

**Build Tool:**
- `univer-cli` - Internal CLI for building packages
- Packages use `univer-cli build` to compile

## Testing

- **Unit Tests:** Vitest, located in each package
- **E2E Tests:** Playwright in `/e2e` directory
- Test coverage is required for all new code
- When adding new plugins, update `vitest.workspace.js`
- Visual regression tests use Playwright snapshots

## Development Workflow

1. All commits must pass ESLint and tests
2. Pre-commit hooks run lint-staged
3. Use conventional commit format
4. CI runs on dev branch with preview deployments
5. When updating UI, you may need to update visual snapshots via GitHub Action

## Deprecating APIs

1. Mark as deprecated in JSDoc with `{@link}` to new API
2. Call `deprecate()` on `ILogService` to log deprecation message
3. Remove in next minor version (or major for heavily-used APIs)

## Additional Resources

- Architecture docs: [docs/ISOMORPHIC.md](docs/ISOMORPHIC.md)
- Contributing guide: [CONTRIBUTING.md](CONTRIBUTING.md)
- Facade API contribution: [docs/CONTRIBUTING-FACADE.md](docs/CONTRIBUTING-FACADE.md)
- Naming conventions: [docs/NAMING_CONVENTION.md](docs/NAMING_CONVENTION.md)
- Community: Discord, GitHub Discussions, Stack Overflow (tag: `univer`)
