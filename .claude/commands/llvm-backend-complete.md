# LLVM Backend Completion - Ralph Wiggum Loop

## Objective
Achieve 95%+ build success rate for the mdhavers LLVM backend (129+/136 files).

**Current Status:** 68/136 (50%)
**Target:** 129/136 (95%)

## Test Command
```bash
find examples -name "*.braw" -exec sh -c './target/release/mdhavers build "$1" >/dev/null 2>&1 && echo "PASS"' _ {} \; 2>/dev/null | grep -c "PASS"
```

## Categorized Remaining Issues

### Phase 1: Callable Fields (HIGH IMPACT - ~8 files)
Files failing with "Method 'X' not found" where X is actually a field containing a callable:
- `thunk` - in promise.braw, promise_demo.braw
- `reducer` - in store.braw, store_demo.braw
- `on_enter_callback`, `on_exit_callback`, `on_transition` - in adventure.braw, fsm_demo.braw, highland_adventure.braw
- `before_each_fn`, `after_each_fn` - in test_framework_demo.braw

**Fix:** In `compile_method_call`, when no method is found:
1. Check if object has a field with that name via `inline_get_field`
2. If field exists, get its value and call it as a function
3. This handles patterns like `masel.callback()` where `callback` is a stored lambda

### Phase 2: Scots Set/Creel Builtins (~6 files)
Missing Scots-dialect set operations:
- `toss_in(set, item)` - add item to set (alias for set.add or dict key insert)
- `heave_oot(set, item)` - remove item from set
- `is_in_creel(set, item)` - check if item in set (alias for contains)
- `empty_creel()` - create empty set (return `{}`)
- `make_creel(items)` - create set from list

**Implementation:** Add these as builtin dispatches in `compile_call`, mapping to existing dict/contains operations.

### Phase 3: File I/O Builtins (~4 files)
- `file_exists(path)` - check if file exists (use libc `access` with F_OK=0)
- `slurp(path)` - read entire file to string (use runtime function)
- `scrieve(path, content)` - write string to file (use runtime function)
- `lines(path)` - read file as list of lines

**Implementation:**
1. Add `__mdh_file_exists`, `__mdh_slurp`, `__mdh_scrieve` to runtime/mdh_runtime.c
2. Declare these in codegen and add builtin dispatches

### Phase 4: Missing Math/Utility Builtins (~5 files)
- `average(list)` - compute average of numeric list
- `sum` or `sumaw` - ensure it works on lists
- `tae_binary(n)` - convert int to binary string
- `pair_up(list1, list2)` - zip two lists together
- `tak(list, n)` - take first n elements (alias for slice)

### Phase 5: Logging/Debug Builtins (~3 files)
- `get_log_level()` - return current log level
- `set_log_level(level)` - set log level
- `blether_format(fmt, ...)` - formatted print (sprintf-like)
- `stacktrace()` - return stack trace string (can return placeholder)

### Phase 6: Spread Operator Enhancement (~2 files)
Current error: "Spread operator can only be used inside list literals"

**Fix:** Handle spread in:
1. Function call arguments: `foo(...args)`
2. List concatenation contexts

### Phase 7: Test Framework Internals (~3 files)
- `__current_suite` - global test suite tracker
- `skip(reason)` - skip test with reason
- `_tick_counter`, `_msg_counter` - internal counters

**Implementation:** These can be simple global variables initialized at startup.

### Phase 8: Scots Word Aliases (~5 files)
Map Scots words to existing builtins:
- `wheesht` -> already exists (trim)
- `slainte` -> greeting/init function (return nil or print)
- `scots_greetin` -> error handler (return error string)
- `braw_time` -> time function (return current time)
- `wee` -> small/min function
- `och` -> error/warning print
- `chynge` -> change/replace in string

## Implementation Strategy

Work through phases in order. For each phase:
1. Identify the specific files that will be unblocked
2. Implement the minimal fix
3. Run test to verify progress
4. Move to next phase

## Key Files to Modify

1. **src/llvm/codegen.rs** - Main codegen, add builtins and fix callable fields
2. **runtime/mdh_runtime.c** - Add file I/O and utility runtime functions
3. **runtime/mdh_runtime.h** - Declare new runtime functions

## Code Patterns

### Adding a new builtin (in compile_call match):
```rust
"builtin_name" => {
    if args.len() != N {
        return Err(HaversError::CompileError("builtin_name expects N arguments".to_string()));
    }
    let arg1 = self.compile_expr(&args[0])?;
    // ... compile other args
    return self.inline_builtin_name(arg1, ...);
}
```

### Adding runtime function:
```c
// In mdh_runtime.c
MdhValue __mdh_function_name(MdhValue arg1, ...) {
    // Implementation
    return result;
}
```

```rust
// In codegen.rs - declare the function
let fn_type = self.types.value_type.fn_type(&[self.types.value_type.into()], false);
let func = self.module.add_function("__mdh_function_name", fn_type, Some(Linkage::External));
```

### Callable field pattern fix (in compile_method_call):
```rust
// After "Method not found" error, before returning error:
// Try to get the field and call it
if let Ok(field_val) = self.inline_get_field(instance, method_name) {
    // field_val might be a callable - try to call it
    let mut call_args: Vec<BasicMetadataValueEnum> = vec![];
    for arg in args {
        call_args.push(self.compile_expr(arg)?.into());
    }
    // Call the field value as a function
    return self.call_value(field_val, &call_args);
}
```

## Completion Promise
"I will achieve 95%+ LLVM backend build success by systematically implementing all missing features."

## Progress Tracking
After each phase, report:
- Files now passing
- New total: X/136 (Y%)
- Remaining blockers
