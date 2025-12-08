---
active: true
iteration: 1
max_iterations: 50
completion_promise: "fib(35)"
started_at: "2025-12-07T21:47:34Z"
---

Extend LLVM backend and optimize performance per wrk_docs/2025.12.07 - ralph prompt -
  llvm backend extension and optimization.md. Phase 1: Add tae_string/tae_int/tae_float to codegen.rs using
  snprintf. Phase 2: Add LLVM optimization passes to compiler.rs using PassManager. Phase 3: Add type specialization
   for int arithmetic. Phase 4: Add tail call optimization for recursive functions. Test each phase with fib(35)
  timing. All benchmarks/mdhavers/*.braw must compile natively. Success when fib(35) < 100ms (currently 166ms) and
  all benchmarks run natively.
