# TypeScript Conventions

- **Build**: tsup (CJS + ESM + .d.ts dual output)
- **Test**: vitest
- **Workspace**: npm workspaces (20 @aix packages)
- **ID generation**: `generateId()` from `@aix/shared`
- **Error classes**: `Object.setPrototypeOf(this, new.target.prototype)` in constructors
- **Templates**: `Object.freeze()` — clone before mutation
- **Type imports**: always from `@aix/shared` — never re-define kernel enums in TS packages
- **@aix/core**: all packages must handle unavailability with `{ status: 'unavailable' }` pattern
- **DeployError**: extends `AixError` with -36xxx code range (ADR-029)
- **Typecheck**: `npx tsc --noEmit` must pass for all 20 packages
