# Native runtime migration to Rust

This document records the contracts for an incremental migration of the native
runtime. The starting audit used commit
`f81ee851563c450e590a57084e5088ffc61de763`. The `.braw` standard library and the
interpreter are separate implementations and remain behavior references.

## First slice: immediate value construction and tag inspection

`runtime/mdh_runtime_core` is a dependency-free, `no_std` Rust crate. It owns the
Rust definition of `MdhValue`, its tag constants, and these C ABI symbols:

| Symbol | Result | Ownership and failure behavior |
| --- | --- | --- |
| `__mdh_make_nil()` | Tag 0, data 0 | Immediate; no allocation or failure |
| `__mdh_make_bool(bool)` | Tag 1, data 0 or 1 | Immediate; input follows C `_Bool` / Rust `bool` validity rules |
| `__mdh_make_int(int64_t)` | Tag 2, unchanged signed data | Immediate; all 64-bit integer values accepted |
| `__mdh_make_float(double)` | Tag 3, unchanged IEEE-754 binary64 bits in `data` | Immediate; no numeric conversion, allocation, or failure |
| `__mdh_get_tag(MdhValue)` | The input's raw one-byte tag | Borrowed observation; no pointer dereference, retain, or release |

The existing `mdh_runtime_rs` static library includes the core crate and uses its
constructors directly. C and Rust callers keep the same symbol names and
declarations; generated LLVM code builds these values inline and does not call
them. The corresponding C definitions are removed; no compatibility
stub should provide a second implementation. Standalone C regression drivers
can compile the same core source to a Rust object and link it without building
the networking, audio, or graphics runtime.

The browser build (`--target wasm32-emscripten`, `src/llvm/wasm.rs`) is the one
place the symbols are not plain: compiled code and the C runtime use the wasm
multivalue ABI there, Rust the standard one. On Emscripten the core exports each
function as `<name>__rsabi`, and `runtime/platform/wasm_abi.c` defines the plain
names over them. `tests/llvm_wasm_target.rs` runs that build under node, and
`runtime_c_standards` compiles the bridge against the shared header on any host.

These functions cannot raise a language error or call back into C. They need no
`RustFrames` guard because their complete call graph contains no raising call,
allocator, callback, or destructor. Keep that property explicit when changing
them. An ordinary C ABI entry must also never let a Rust panic escape.

This small slice deliberately leaves arithmetic, comparisons, truthiness, and
allocation in their existing implementations. In particular, `__mdh_truthy` is
not a pure tag test: its dict/set path calls `__mdh_kv_payload_ptr`, which can
compact tombstones, allocate, and raise an out-of-memory error. Moving that
path into Rust unchanged could introduce a `longjmp` across Rust frames.

## ABI contract

The three representations that must agree are:

- C: `MdhValue` in `runtime/mdh_runtime.h`.
- Rust: `#[repr(C)] MdhValue` in `runtime/mdh_runtime_core`.
- Generated LLVM IR: the unpacked `{ i8, i64 }` in `src/llvm/types.rs`. Codegen
  builds the five migrated values inline; the `src/llvm/runtime.rs`
  declarations of them are used only by that file's layout test. The layout
  still matters because every other runtime call passes and returns `MdhValue`.

On the currently assumed 64-bit layouts, `MdhValue` has size 16, alignment 8,
`tag` at offset 0, and `data` at offset 8. The intervening bytes are padding,
not part of the value. Do not pack the structure, compare it with `memcmp`,
serialize its raw bytes, or give its padding a semantic meaning. Layout checks
and an actual C-to-Rust call test are separate requirements: equal size and
offsets do not prove identical argument and return-value calling conventions.

The tag remains a raw `u8`, not a Rust enum with invalid discriminant values.
The existing values are nil 0, bool 1, int 2, float 3, string 4, list 5, dict 6,
function 7, class 8, instance 9, range 10, set/creel 11, closure 12, bytes 13,
and native object 14. `__mdh_get_tag` preserves even an unrecognized byte; it
does not validate or dereference the payload.

`data` is a signed 64-bit storage word. Integers use it directly; booleans made
by the constructor use 0 or 1; floats use `to_bits()` / `from_bits()`, preserving
negative zero, infinities, and NaN bit patterns. Pointer-bearing values store
the pointer in that word. A string points directly to NUL-terminated character
data, not to the `MdhString` structure declared in the header. A list points to
`MdhList`; bytes point to `MdhBytes`; dictionaries and sets point to private
handles. Do not infer a common payload structure from the shared outer value.

`Copy` and `Clone` on this ABI structure copy bits only. They do not retain a
heap object or transfer an ownership obligation. A Rust wrapper that owns a
runtime reference must be a separate type with an explicit contract.

`MdhRsResult` is also shared between the public C header and the Rust core. It
contains a one-byte `ok` status followed by `MdhValue value` and `MdhValue error`.
On the same 64-bit layout its size is 40, alignment is 8, and field offsets are
0, 8, and 24. The status is 0 or 1; the inactive value/error field is nil. Keep
its per-entry ownership contract distinct from the layout contract. C test
fixtures should include the header instead of redeclaring this structure.

## Allocation and ownership contract

`runtime/gc_stub.c` is the active allocator and reference-count implementation,
despite its historical filename. It is not a removable collector dependency.

- Runtime objects use `__mdh_obj_alloc(kind, size)` and related object helpers.
  With counting enabled, their payload follows a 16-byte `MdhObjHeader`
  containing `i64 refcount`, `u32 kind`, and `u32 flags`; a new object starts at
  reference count 1. With counting disabled, allocation can be headerless.
- Object-owned buffers use `__mdh_buf_alloc`, `__mdh_buf_realloc`, and
  `__mdh_buf_free`. A buffer is not itself an `MdhValue` payload. Its owning
  object's cleanup releases it through `__mdh_rc_free_buffers`.
- Runtime references must not be created with `Box`, `Vec`, `CString::into_raw`,
  or a second allocator and then freed by the C allocator. Rust's temporary
  containers may use Rust allocation, but converting them into mdhavers values
  must copy into or explicitly transfer compatible runtime-managed storage.
- Counted heap tags are selected by `MDH_RC_TAG_MASK` / `RC_HEAP_TAG_MASK`
  (`0x7A70`). Immortals are never changed by retain/release. The magic and DEAD
  checks are part of the debugging contract, not spare header bits.
- At zero references, ordinary counted objects release their children and
  buffers and can be freed or cached. `MDH_RC_NOFREE=1` instead keeps DEAD
  objects for diagnostics. Native objects are excluded from ordinary freeing
  because other runtime state keeps address-based handles to them.
- Once threads exist, the allocator uses atomic count paths and shared-slot
  publication rules. Preserve `__mdh_rc_fast`, `__mdh_line_threaded`, and the
  shared load/store behavior when migrating any part of reference counting.

The general convention is borrowed arguments and owned newly created results,
but it is not a universal rule for every legacy function. Audit each function
and every caller before migrating it:

| Existing interface | Ownership details |
| --- | --- |
| Immediate constructors in this slice | No heap references are acquired or consumed |
| `__mdh_make_string_try`, `__mdh_make_list_try`, `__mdh_bytes_new_try`, `__mdh_empty_dict_try` | Success returns a new owned value through `out`; failure reports status and leaves nil |
| `__mdh_list_push_owned_try` | Consumes the supplied value on success and failure; failure releases it |
| `__mdh_dict_set_owned_try` | Borrows the dictionary; consumes key and value on both success and failure |
| `__mdh_dict_get`, `__mdh_dict_get_default` | Return a borrowed entry or borrowed default, without retain |
| `__mdh_dict_set` and its owned variant | Mutate and return the same dictionary alias; the return does not acquire another dictionary reference |
| `__mdh_get_last_error` | Returns the thread's stored error reference as a borrowed value |
| `__mdh_call_fn_caught` | Reports status and returns an owned result or a retained error through `out` |
| `__mdh_shared_load`, `__mdh_shared_store` | Load returns an owned reference; store consumes the caller's reference and releases the old value after publication |

“Borrowed” says nothing about whether a function can raise. For example,
`__mdh_dict_get_default` raises on an invalid dictionary tag. Existing Rust
callers must establish its preconditions. A future safe Rust facade should
make that validation explicit rather than treating all existing externs as
unconditionally safe or non-raising.

## Errors and the Rust frame boundary

The language's C exception path uses a per-thread handler stack and `longjmp`.
A Rust frame must not be skipped by that jump. `catch_unwind` handles Rust
panics; it does not make C `longjmp` safe.

The existing bridge is the pattern for later migrations:

1. Rust calls non-raising status-returning helpers, such as the `_try`
   allocation/container functions and `__mdh_to_string_checked`. It must not
   use `__mdh_obj_alloc_checked`, `__mdh_raise_out_of_memory`, or other raising
   helpers underneath its frames.
2. Rust drops temporary objects and outstanding borrows normally. Higher-level
   entries use `MdhRsResult { ok, value, error }`; `mdh_finish` converts a noted
   allocation failure to the preallocated immortal out-of-memory error.
3. The C wrapper receives that result after the Rust call returns, leaves its
   Rust-frame guard, and only then raises a language error.
4. A callback into generated code must use a C catch trampoline such as
   `__mdh_call_fn_caught`, whose handler is inside the Rust call. It returns an
   explicit status rather than jumping past the Rust caller.

Debug runtime builds use `MDH_ALLOC_FAULTS`, `MDH_IN_RUST`, and
`__mdh_rust_enter` / `__mdh_rust_leave` to abort when a raise would cross a
marked Rust frame. Direct audio/3D entries use `RustFrames`. The guard is a
diagnostic, not the safety mechanism: release builds still require the same
non-raising call graph. Do not move a raising C body into an `extern "C"`
Rust function merely because it has no obvious Rust local allocations.

The existing media bridge is not uniformly converted to the status-returning
pattern above. For example, `audio_stub::hurl` calls C `__mdh_hurl` directly
without a frame guard; `audio::with_state` drops its guard and local resources
before calling `__mdh_hurl` from Rust. These pre-existing paths need a separate
boundary audit and conversion before claiming that every Rust entrypoint
returns to C before a raise. This scalar migration does not change them.

## Current dependency boundaries

| Component | Depends on / calls | Role |
| --- | --- | --- |
| `build.rs` and `cc` | C compiler and archiver; nested Cargo invocation | Build tooling |
| mdhavers LLVM compiler | LLVM 19 / inkwell; host target; system link driver | Produces native object code and links an executable |
| Generated code | C runtime, Rust runtime archive, allocator, selected screen/sound archives, system libraries | Native executable |
| `mdh_runtime.c` | Core constructors, higher-level Rust helpers, allocator and platform functions | Remaining value/collection/string/arithmetic/error behavior |
| `mdh_runtime_rs` | `mdh_runtime_core`; C allocation, container, formatter, and error helpers; Rust and native dependency crates | JSON, regex, networking, optional audio/3D |
| `mdh_runtime_core` | Rust `core` only | This migration's five leaf symbols and shared Rust value definition |
| `gc_stub.c` | Runtime child enumeration, buffer cleanup, current line, OOM error, threaded state | Allocation and reference counting |
| `.braw` modules | Language-level functions and builtins | Standard-library source, distinct from the native implementation |

The runtime archives call into each other; `src/llvm/compiler.rs` already groups
them for ELF linkers. Keep C fixtures and production linkage using the same
symbol implementation. Inspect the final archive for one definition of each
migrated symbol and inspect the core object for unexpected external calls.

This slice removes five C implementations and their Rust-to-C declarations.
It does not make `cc`, the C compiler, the allocator, platform C, or any of the
remaining archives redundant. Remove a build dependency only when every
remaining source consumer is gone.

Rust source is not a guarantee of a C-free dependency graph. The current Rust
runtime unconditionally depends on `libsrtp`, `udp-dtls`, and vendored OpenSSL;
optional audio also includes a native miniaudio implementation. Vendoring can
move a dependency into a static archive without eliminating its C code or
build tools. Generated executables also retain platform libraries/CRT
dependencies. LLVM is used by the compiler; it is not automatically a linked
dependency of every generated program. Use an executable dependency inspection
(`readelf`, `otool`, or platform equivalent) to establish the actual result.

Cargo's `build.rs` is the supported build path used here. The old
`runtime/Makefile` still describes a C-only/Boehm build and omits the modern
Rust/platform pieces; it is not a complete standalone runtime build. Do not
use that legacy file as the dependency inventory for generated executables.

## Repeatable method for the next component

1. Select a complete, narrow group of symbols and inventory both direct calls
   and callbacks. Classify each path as pure, borrowing, retaining, consuming,
   allocating, mutating, or potentially raising.
2. Write down the exact observable behavior, ABI layout and calling convention,
   aliasing rules, result ownership, failure cleanup, and error text. Check
   both interpreter/native parity and the C implementation; do not assume
   either alone describes all compatibility obligations.
3. Prefer pure helpers in the core crate. For allocating or failing work, use
   the established allocator and explicit status/results. Keep the final
   raising wrapper in C until the language's exception mechanism changes.
4. Implement the Rust symbols, remove their C definitions, and update all
   production and standalone-test linkage in the same change. Register newly
   added source/manifests with the outer build's change tracking.
5. Verify C/Rust calls in both directions when relevant, value field offsets,
   symbol uniqueness, and no unexpected core dependencies. Compare fields and
   float bits, never padding. Cover minimum/maximum integers, both zeros,
   infinities, subnormals, and NaN payloads where floats cross the ABI.
6. Run the relevant interpreter/native parity and regression suites with
   reference counting on and off. For heap operations also test allocation
   failure, DEAD-object detection, immortals, aliasing, and threaded paths;
   retain the existing Rust-frame-error regression checks.
7. Record precisely which platforms were compiled, executed, or only inspected.
   Remove libraries/tooling only after checking remaining users and the final
   executable's actual imports. Keep deferred changes explicit.

Useful existing regression suites include `backend_parity`,
`llvm_runtime_rs_tests`, `llvm_runtime_raise_safety`,
`llvm_safety_regressions`, `llvm_rc_runtime`, `llvm_rc_intern_runtime`,
`llvm_rc_terminal_runtime`, `runtime_c_chunks_overflow`, and
`runtime_dtls_fd_ownership`. The dedicated core tests and cross-language ABI
test cover the five migrated leaf functions directly; generated code builds
ordinary literals inline and never calls them, so compiled programs would not
prove those exported functions work. Their real callers are
`runtime/mdh_runtime.c` and `runtime/mdh_runtime_rs`.

## Windows and future SPARC work

The core slice has no OS calls, pointer dereferences, allocator, inline assembly,
or endian-specific byte manipulation. That makes it a useful portability test
unit, not evidence that the whole native backend already supports a target.

Existing Windows obstacles outside this slice include unconditional
`std::os::unix::io::FromRawFd` in the higher-level runtime and Unix descriptor
ownership in TLS/DTLS paths. Windows socket handles need their own APIs and
width/ownership audit. The build has both MSVC and GNU-style archive naming
concerns, while executable linkage currently uses a `cc` driver and Unix-style
library flags. C sources also contain compiler-specific constructs. Record
these as existing platform work; do not equate a successful Windows core
cross-compile with a complete Windows runtime test.

For SPARC, choose the actual ABI first: 64-bit SPARC V9 and a 32-bit SPARC
target are different ports. The current backend initializes the native target,
uses the host triple and data layout, and embeds runtimes built for that host.
Cross-compilation needs coordinated target selection, available LLVM target
support, a target Rust toolchain/core library, target-built runtime archives,
system libraries, and a target linker/sysroot.

Additional gates are:

- Prove C, Rust, and generated LLVM argument/return lowering for aggregates and
  booleans. `#[repr(C)]` fixes field layout; it does not automatically rewrite
  hand-declared LLVM function signatures to every target's C ABI. The five
  migrated symbols are not called from generated code, but every other runtime
  call passes `MdhValue`, so this gate still applies.
- Audit hard-coded 16-byte values, alignment, pointer-to-`i64` conversions, and
  pointer-sized private handles. The Rust JSON walker currently reads dict
  handle fields through `*const i64`, whereas C uses `intptr_t`: that assumption
  requires explicit attention on a 32-bit port.
- Exercise a big-endian target. `to_bits()` / `from_bits()` preserve the numeric
  bit pattern on either endian; raw structure serialization, external packet
  layouts, and field offsets still require independent review.
- Revalidate reference-count atomics, TLS, stack alignment/probes, signal stack
  handling, and the target `jmp_buf` size/alignment. The existing 512-byte
  handler reservation is checked by C, not a universal cross-target promise.
- Audit platform availability of each optional and unconditional dependency.
  LLVM code generation for an ISA alone does not provide those runtime ports.

The next low-risk additions are other non-allocating tag predicates, after
checking their exact language behavior. Arithmetic should follow only with
explicit checked-overflow and error-result boundaries; strings, containers,
reference counting, I/O, and the exception mechanism need separate contracts.

## Reproducing the checks

Run the C/Rust ABI probe without LLVM or the higher-level runtime dependencies:

```sh
cargo test --no-default-features --test runtime_core_abi
cargo clippy --manifest-path runtime/mdh_runtime_core/Cargo.toml --all-targets -- -D warnings
```

With LLVM 19 installed (set `LLVM_SYS_191_PREFIX` when it is not found
automatically), add the compiler's layout/tag comparison and run the existing
behavior and ownership regressions:

```sh
cargo test --no-default-features --features cli,llvm,native \
  --test runtime_core_abi --test backend_parity --test golden_tests \
  --test llvm_runtime_rs_tests --test llvm_runtime_raise_safety \
  --test llvm_safety_regressions --test llvm_rc_runtime \
  --test llvm_rc_intern_runtime --test llvm_rc_terminal_runtime \
  --test runtime_c_chunks_overflow --test runtime_dtls_fd_ownership
cargo test --no-default-features --features cli,llvm,native --lib llvm::ownership::tests
cargo check --locked --manifest-path runtime/mdh_runtime_rs/Cargo.toml \
  --features audio,graphics3d
```

The existing intern/terminal fixtures invoke `deltic timeout`; provide that
tool or an equivalent local adapter to the system `timeout` command. DTLS
fixtures need permission to create local UDP sockets. A sandbox denying
`socket()` is not a valid run of the descriptor-ownership tests. For the
strict C17 syntax regression, use Clang for `runtime_c_standards` (it otherwise
skips); selecting `CC` for the Cargo build also affects all production C files.

`tests/support/runtime_core.rs` is a **host** test helper: it compiles the core
with the host Rust toolchain. For cross-target checks, use the same portable C
fixture with an explicit target, target C compiler and, when available, a runner.
For example, after installing the SPARC64 Rust target, C cross-toolchain and QEMU:

```sh
mkdir -p /tmp/mdh-core-portability
rustc runtime/mdh_runtime_core/src/lib.rs --edition=2021 \
  --crate-name=mdh_runtime_core --crate-type=rlib --emit=obj \
  -C opt-level=2 -C panic=abort --target sparc64-unknown-linux-gnu \
  -o /tmp/mdh-core-portability/sparc64.o
sparc64-linux-gnu-gcc -std=c17 -Wall -Wextra -Werror -O2 -static -Iruntime \
  tests/fixtures/runtime_core_abi.c /tmp/mdh-core-portability/sparc64.o \
  -o /tmp/mdh-core-portability/sparc64-abi
qemu-sparc64 /tmp/mdh-core-portability/sparc64-abi
```

For Windows GNU, use `--target x86_64-pc-windows-gnu` for the Rust object
and `x86_64-w64-mingw32-gcc` for the fixture. Compile an MSVC object separately
with `--target x86_64-pc-windows-msvc`; execution/link validation there requires
the corresponding Windows C toolchain and CRT. The MSVC core object references
`_fltused`, illustrating why no Rust crate dependencies does not mean no CRT
requirements. The native test's MSVC driver path uses `cl` (or `CC=clang-cl`)
from a configured developer environment.

Inspect both production archives with `nm --defined-only`: each migrated
symbol must have exactly one definition in `mdh_runtime_rs.a` and none in
`mdh_runtime_c.a`. Inspect the standalone core with `nm -u` (or the target
equivalent), and inspect an actual generated executable's imports separately.

Reference material: [Rust C layout](https://doc.rust-lang.org/reference/type-layout.html#the-c-representation),
[Rust FFI boundaries](https://doc.rust-lang.org/nomicon/ffi.html#ffi-and-unwinding),
and [Rust target support](https://doc.rust-lang.org/rustc/platform-support.html).

## Validation record: 31 October 2026

The migration was built on Linux x86-64 using Rust 1.99.1, LLVM 19.1.1 and
GCC 13.3. The relevant production feature set was `cli,llvm,native`, with
debug allocation-fault and Rust-frame diagnostics enabled.

| Check | Observed result |
| --- | --- |
| C/Rust ABI without LLVM enabled | 1 test passed |
| C/Rust ABI plus LLVM native layout/tags | 2 tests passed |
| Runtime ownership declaration/definition table | 5 tests passed (covers the existing ownership table; the five migrated symbols have no entry in it) |
| Existing `backend_parity` | 109 tests passed |
| Existing interpreter golden suite | 268 cases passed; 2 skipped by the existing harness |
| Existing native golden suite | 205 cases passed; 65 skipped by the existing harness |
| Existing Rust JSON/regex runtime tests | 5 tests passed |
| Existing `llvm_runtime_raise_safety` | 11 tests passed, including its RC-on/off and allocation-failure cases |
| Existing `llvm_safety_regressions` | 28 tests passed |
| Allocator/reference-count C runtime tests | 12 tests passed |
| Interning/terminal C runtime tests | 5 + 1 tests passed |
| C chunks-overflow and DTLS descriptor ownership | 1 + 2 tests passed |
| Strict C17 regression under Clang 19 | Passed without skipping |
| Core Clippy with warnings denied | Passed |
| Higher-level runtime with `audio,graphics3d` | `cargo check --locked` passed; devices were not exercised |
| Production archive definitions | Exactly one definition per migrated symbol in the Rust archive; none in the C archive |
| SPARC64 Linux C/Rust fixture | Cross-compiled, statically linked and executed successfully with QEMU SPARC64 |
| Windows GNU x86-64 C/Rust fixture | Cross-compiled and linked successfully; not executed |
| Windows MSVC x86-64 core | Object compiled successfully; C linkage/execution not tested |

The standalone optimized core objects had no undefined symbols on Linux,
Windows GNU and SPARC64. The MSVC object referenced `_fltused` only. A generated
Linux executable exercising JSON-created scalars, regex booleans, a caught
regex error and list length matched the interpreter output. Its ELF dynamic
imports were `libm.so.6`, `libc.so.6`, and `ld-linux-x86-64.so.2`; it had no
LLVM shared-library import. This sample is evidence for that executable, not
a dependency guarantee for every feature combination.

Two environment issues were resolved before counting the tests as passing:
the vendored OpenSSL build hit a directory-cleanup error in the synchronized
working directory, so build output was moved to a temporary target directory;
DTLS fixtures initially could not create sockets in the sandbox, so they were
rerun with local-socket access. The existing `deltic timeout` calls used a
temporary adapter to GNU `timeout`. The production build still reports existing
C warnings in dictionary removal and the Unix socket wrapper; these were not
introduced or suppressed by this migration.

C calls to migrated constructors now cross a compiled-object boundary: built at
-O2, mdh_runtime.c went from 0 to 309 out-of-line calls to them, where it used to
inline them. Measured on an Apple Silicon Mac (2026-11-02, native release builds
of trunk and this slice, interleaved runs), the cost does not show: a 120M
iteration int/float loop ran at 1.01x of trunk (median 0.994s against 0.986s), and
the Bogle Castle campaign replay at 1.00x (2.925s against 2.928s), with identical
output. One host and a few percent of noise; do not claim a performance
improvement from this slice either.
