fix(ot): implement LWW strategy for SetRangeValues to ensure client consistency

## Problem
When multiple clients concurrently modify the same cell, the SetRangeValues 
transform returned operations unchanged, violating OT consistency and causing 
client states to diverge.

## Solution
Implemented Last-Writer-Wins (LWW) conflict resolution strategy:

### 1. SetRangeValues Transform (Rust)
- When two operations modify the same cell, m2 (server-first) wins
- m1 removes conflicting cells to become NOOP for those cells
- Ensures OT consistency: $State + m1 + m2' = $State + m2 + m1'

### 2. Client Logic Documentation (TypeScript)
- Confirmed existing "coordinate realignment" strategy is correct
- Added comprehensive documentation explaining the rebase approach
- No logic changes needed - implementation was already correct

## Testing
- Added 3 new test cases for conflict scenarios
- Added 1 comprehensive OT consistency test
- All 49 tests passing

## Files Changed
- `packages/univer-ot-wasm/src/transform/set_range_values.rs`
- `packages/univer-ot-wasm/src/transform/set_range_values_test.rs`
- `packages/collaboration/src/services/collaboration.service.ts`

## Verification
```
$ cargo test --lib
test result: ok. 49 passed; 0 failed
```

Resolves concurrent editing conflicts with LWW semantics.
