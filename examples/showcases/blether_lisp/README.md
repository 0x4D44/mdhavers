# Blether Lisp - a wee Scheme in mdhavers

A small Lisp interpreter written in mdhavers. It has:

- a tokeniser and reader: nested lists, integers, floats, symbols, strings
  with `\"` and `\n` escapes, `'quote` shorthand, `;` comments, `#t`/`#f`
- environments with lexical scope, and closures with private state
- special forms: `define` (variables and `(define (f args) ...)`), `lambda`
  (fixed or variadic `(lambda args ...)`), `if`, `let`, `begin`, `quote`,
  `set!`, `cond`, `and`, `or`
- proper tail calls: the evaluator is a loop, so a tail-recursive Lisp loop
  does not grow the mdhavers stack
- builtins: `+ - * / modulo = < > <= >= equal? eq? car cdr cons list null?
  length append map filter reduce apply not display error number? symbol?
  list? procedure? abs`
- a prelude written in Blether Lisp: `cadr zero? even? odd? range compose sum`
- error messages in Scots, for example
  `Ye cannae tak the car o' an empty list, ya numpty!`

## Running

```bash
# Built-in demo: factorial, fibonacci, closures, map/filter/reduce,
# quicksort, a 5000-step tail-recursive loop, and some errors
mdhavers run examples/showcases/blether_lisp/blether_lisp.braw

# Run your own Lisp file (an absolute path is safest: the interpreter
# changes into the script directory, the native binary does not)
LISP_FILE=/path/to/prog.scm mdhavers run examples/showcases/blether_lisp/blether_lisp.braw

# Native (LLVM)
mdhavers build examples/showcases/blether_lisp/blether_lisp.braw -o blether_lisp
./blether_lisp
```

The output is a transcript: each form is echoed as `lisp> ...`, followed by
`=> value`, or by `!! message` if it fails. After an error the run carries on
with the next form.

## Timings

The full demo, measured on Apple Silicon (macOS) on 2026-10-23:

| Backend | Wall time |
|---|---|
| Interpreter (debug build of `mdhavers`) | 2.5 s |
| Native (`mdhavers build`, LLVM `-O2`) | 0.01 s |

Both backends produce identical output.

## Notes on the native backend

A few lines are shaped to avoid native (LLVM) backend bugs. Comments in the
source mark each one:

- `atom_shape` is separate from `parse_atom`, and `bind_args` copies its
  parameter to a local and binds the rest list before its loop. A `whiles`
  loop loses a parameter that the loop does not reference directly.
- The comparison builtins use named functions rather than lambdas. Calling a
  lambda that was passed in as a parameter fails natively.
- `PRELUDE` is defined before `make_global_env`. Native code needs a global to
  be defined, in the source text, before any function that uses it.

A cut-down copy is the golden test
`tests/golden/integration/showcase_blether_lisp.braw`.
