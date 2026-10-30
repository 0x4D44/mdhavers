# Standard Library

The standard library is a set of modules written in mdhavers itself, kept in the `stdlib/` directory of the repository (59 modules). Most are pure mdhavers: string and maths helpers, data structures, a test framework, date handling and so on. A few wrap native built-ins (sockets, TLS, threads, terminal, audio, 3D) and only work on backends that provide those built-ins.

Each module has its own page in this section. The page lists every public function and class, with parameters, return values and the Scots error messages they `hurl`, plus one runnable example.

## Importing a module

Use `fetch` with a path. The `.braw` extension is optional:

```text
fetch "stdlib/strings"             # names land in the current scope
fetch "stdlib/strings" tae str     # names land in a dictionary called str
```

With `tae`, call functions through the alias, for example `str["name"]`. See [Modules](../learn/07-modules.md) for the language-level rules.

### How `fetch` finds a module

For a relative path such as `stdlib/json` (or `json`), the interpreter tries these places in order and uses the first that exists:

1. the path relative to the importing file's directory, then relative to the process's working directory;
2. `stdlib/<path>` under the importing file's directory (a leading `lib/` is also stripped, so `fetch "lib/colors"` finds `stdlib/colors.braw`);
3. the same two forms (`<path>` and `stdlib/<path>`) in each parent directory of the importing file, walking up to the filesystem root;
4. next to the `mdhavers` executable (`<exe dir>/<path>`, `<exe dir>/stdlib/<path>`, and `<exe dir>/../lib/<path>`);
5. `$MDHAVERS_HOME/lib/<path>`.

If none exists, `fetch` hurls a module-not-found error. A module that imports itself (directly or through a cycle) is rejected with a circular-import error. A module is loaded once and cached.

Modules are plain source files, so top-level names a module defines (functions, classes and variables) become visible to the importer. Two modules that define the same name will clash; use `tae` aliases to avoid that.

## Backend support

Modules marked **native** in the tables below call built-ins that exist in the interpreter and (mostly) the LLVM backend, but not in the JavaScript or WASM outputs. Everything else is pure mdhavers and portable. Stability notes from `stdlib/README.md`: `prelude`, `strings`, `maths`, `collections`, `structures`, `functional`, `json`, `testing`, `benchmark` and `colors` are treated as stable-ish; the rest are experimental and may change.

## Overlapping modules

Several areas have more than one module. Each page says how they differ; the short version:

| Area | Modules | Prefer |
|------|---------|--------|
| Maths | [maths](./maths.md), [math](./math.md) | `maths` for everyday numbers and statistics; `math` for geometry, vectors and advanced functions |
| Time | [time](./time.md), [datetime](./datetime.md), [dates](./dates.md) | see the pages; `datetime` has the richest classes |
| Testing | [test](./test.md), [testing](./testing.md) | `testing` is the documented, stable-ish one |
| Patterns | [patterns](./patterns.md), [patterns_match](./patterns_match.md) | see the pages |
| Scots | [scots](./scots.md), [scots_lang](./scots_lang.md) | see the pages |
| Networking | [network](./network.md), [network_real](./network_real.md) | `network` is a simulation; `network_real` uses real sockets |
| State machines | [statemachine](./statemachine.md), [fsm](./fsm.md) | see the pages |

## Module list

### Core

| Module | Description |
|--------|-------------|
| [prelude](./prelude.md) | Common helpers loaded for every program |

### Text

| Module | Description |
|--------|-------------|
| [strings](./strings.md) | String utilities |
| [text](./text.md) | Advanced text processing |
| [template](./template.md) | String templating |
| [patterns](./patterns.md) | Simple wildcard pattern matching |
| [patterns_match](./patterns_match.md) | Glob matching, templates and tokenizing |
| [table](./table.md) | Table and tabular data manipulation |

### Data formats and validation

| Module | Description |
|--------|-------------|
| [json](./json.md) | JSON parse and stringify |
| [validate](./validate.md) | Data validators (email, URL, phone and more) |
| [config](./config.md) | Configuration handling (INI, environment, flags) |
| [paths](./paths.md) | Path manipulation (**native** for file checks) |
| [uuid](./uuid.md) | UUID generation |
| [crypto](./crypto.md) | Simple hashing, encoding and checksums |
| [bytes](./bytes.md) | Byte buffers and endian helpers |

### Data structures and functional programming

| Module | Description |
|--------|-------------|
| [collections](./collections.md) | Collection utilities and containers |
| [structures](./structures.md) | Stack, queue, deque, set, priority queue and more |
| [functional](./functional.md) | map, filter, reduce, compose, memoize |
| [store](./store.md) | State management store |
| [matrix](./matrix.md) | Matrix and grid operations |

### Maths and randomness

| Module | Description |
|--------|-------------|
| [maths](./maths.md) | Mathematics utilities |
| [math](./math.md) | Advanced mathematics and geometry |
| [chance](./chance.md) | Random and chance operations |

### Time

| Module | Description |
|--------|-------------|
| [time](./time.md) | Date and time utilities |
| [datetime](./datetime.md) | Date and time handling with classes |
| [dates](./dates.md) | Date utilities |
| [scheduler](./scheduler.md) | Task scheduling |

### Testing and diagnostics

| Module | Description |
|--------|-------------|
| [testing](./testing.md) | Unit-test framework |
| [test](./test.md) | A second testing framework |
| [proptesting](./proptesting.md) | Property-based testing |
| [benchmark](./benchmark.md) | Timing and comparison |
| [logging](./logging.md) | Structured logging |

### Control flow and concurrency

| Module | Description |
|--------|-------------|
| [promise](./promise.md) | Promise, Result and Option patterns |
| [events](./events.md) | Event emitter and pub/sub |
| [event_loop](./event_loop.md) | Socket polling and timers (**native**) |
| [concurrency](./concurrency.md) | Threads, mutexes, channels (**native**) |
| [statemachine](./statemachine.md) | State machine class |
| [fsm](./fsm.md) | Finite state machine |

### Networking and protocols

| Module | Description |
|--------|-------------|
| [network](./network.md) | Simulated networking |
| [network_real](./network_real.md) | Real sockets and DNS (**native**) |
| [http](./http.md) | HTTP client utilities and mock client |
| [sip](./sip.md) | SIP helpers (**native** for DNS) |
| [rtp](./rtp.md) | RTP header and packet helpers |
| [rtcp](./rtcp.md) | RTCP receiver reports |
| [tls](./tls.md) | TLS wrappers (**native**) |
| [srtp](./srtp.md) | SRTP wrappers (**native**) |

### Terminal and command line

| Module | Description |
|--------|-------------|
| [colors](./colors.md) | ANSI colours and styles |
| [tui](./tui.md) | Terminal user-interface widgets |
| [menus](./menus.md) | Interactive menus and forms |
| [cli](./cli.md) | CLI helpers |
| [argparse](./argparse.md) | Command-line argument parsing |

### Graphics, games and fun

| Module | Description |
|--------|-------------|
| [ascii_art](./ascii_art.md) | ASCII art and text graphics |
| [game](./game.md) | ASCII game utilities |
| [adventure](./adventure.md) | Text-adventure engine |
| [quiz](./quiz.md) | Quiz and trivia framework |
| [music](./music.md) | Scottish music and sound (**native** audio) |
| [tri](./tri.md) | 3D module stub for non-native backends |

### Scottish flavour

| Module | Description |
|--------|-------------|
| [scots](./scots.md) | Scottish phrases, sayings and culture |
| [scots_lang](./scots_lang.md) | Scottish language utilities |
| [poetry](./poetry.md) | Scottish poetry and verse generation |
