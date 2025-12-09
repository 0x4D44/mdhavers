---
active: true
iteration: 1
max_iterations: 20
completion_promise: null
started_at: "2025-12-09T21:34:45Z"
---

Implement full class/OOP support in the LLVM backend following the detailed specification
   in wrk_docs/2025.12.09 - ralph prompt - llvm class support.md. This includes: (1) Stmt::Class compilation, (2)
  Expr::Get for property access, (3) Expr::Set for property assignment, (4) Expr::Masel for self-reference, (5)
  method calls with masel binding, (6) class instantiation. Test by building games/tetris/tetris.braw. Do NOT
  implement superclass/inheritance support yet - focus on single classes only.
