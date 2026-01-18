# Code Simplification Plan

## Overview
Systematic refactoring to reduce code duplication and improve maintainability.
Originally estimated total savings: 300-400 lines of production code.
**Actual savings: ~119 lines** (Phase 1.1 + 1.2 on HOF functions). Other phases had lower ROI than estimated due to special cases and existing helpers.

## Priority Order

### Phase 1: High-Impact Helper Extraction

#### 1.1 Arity Checking Helper (Quick Win)
- **Location:** `src/interpreter.rs`
- **Problem:** 15+ identical arity check patterns
- **Solution:** Extract `check_arity()` helper function
- **Estimated savings:** 10-15 lines
- **Status:** [x] Complete - 56 lines removed (9 functions updated)

#### 1.2 List Extraction Helper
- **Location:** `src/interpreter.rs`
- **Problem:** Same 5-7 line pattern appears 10+ times
- **Solution:** Extract `extract_list()` helper
- **Estimated savings:** 30-50 lines
- **Status:** [x] Complete - 63 lines removed (9 functions updated)

#### 1.3 HOF Boilerplate Reduction
- **Location:** `src/interpreter.rs` lines ~12,268-12,543
- **Problem:** 9 HOF functions share 90%+ similar code
- **Solution:** Extract helper for arity + list extraction combo
- **Estimated savings:** Originally 120-150 lines, but Phase 1.1 and 1.2 captured most savings
- **Status:** [x] Subsumed by Phase 1.1 and 1.2 - remaining boilerplate is minimal

### Phase 2: Arithmetic Simplification

#### 2.1 Numeric Binary Op Helper
- **Location:** `src/interpreter.rs` `binary_op` function
- **Problem:** Repeated numeric type dispatch pattern
- **Solution:** Extract `numeric_binary_op()` helper
- **Estimated savings:** Originally 80-120 lines
- **Status:** [x] Skipped - special cases (string concat, list concat, string repeat, zero checks) make extraction complex with low ROI

### Phase 3: Property Access Consolidation

#### 3.1 Get Property Helper
- **Location:** `src/interpreter.rs`
- **Problem:** Deeply nested matches for property access
- **Solution:** Extract `get_property()` function
- **Estimated savings:** Originally 25-35 lines
- **Status:** [x] Skipped - `dict_get()` helper already exists at line 2040

#### 3.2 Bind Self Helper
- **Location:** `src/interpreter.rs`
- **Problem:** Same 15-line pattern in 4 locations
- **Solution:** Extract `bind_self()` helper
- **Estimated savings:** Originally 20-25 lines
- **Status:** [x] Skipped - only 3 occurrences (~5 lines each), extraction complexity outweighs ~12 line savings

### Phase 4: Future Improvements (Lower Priority)

- RefCell helper methods on Value types
- WASM compiler import generation
- Set operation consolidation in value.rs
- Generic statement walker in compiler.rs

## Progress Tracking

- [x] Phase 1.1 - Arity checking
- [x] Phase 1.2 - List extraction
- [x] Phase 1.3 - HOF consolidation (subsumed by 1.1+1.2)
- [x] Phase 2.1 - Numeric binary ops (skipped - low ROI)
- [x] Phase 3.1 - Get property (skipped - helper exists)
- [x] Phase 3.2 - Bind self (skipped - low ROI)

## Validation

After each phase:
1. `cargo fmt`
2. `cargo clippy -- -D warnings`
3. `cargo test` (if tests exist for modified code)
