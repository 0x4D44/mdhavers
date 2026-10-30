# logging

Structured logging with levels. A `Logger` filters by minimum level, formats lines with an optional timestamp, ANSI colour and Scots level names, and passes each emitted line to handler functions. The module also has a `LogBuffer` to collect entries, a `StructuredLogger` that prints JSON lines, a `PerfLogger` for timing named operations, and a lazily created global logger with `log_*` shortcuts.

Import: `fetch "stdlib/logging"`

Pure mdhavers; runs on every backend that supports `noo()` and `json_stringify`. Loading the module prints `Logging module loaded! Ready tae keep track o' things!`. Output goes to standard output with `blether`; there is no file output, so use a handler if you want to store lines. Timestamps are `noo()` in milliseconds since the epoch. Helpers starting with `_` (`_global_logger`) are internal and not documented.

## Constants

| Name | Value |
| --- | --- |
| `LOG_DEBUG`, `LOG_INFO`, `LOG_WARN`, `LOG_ERROR`, `LOG_FATAL` | 0, 1, 2, 3, 4 |
| `LEVEL_NAMES` | `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL` |
| `LEVEL_SCOTS` | `HAVERS`, `KEN`, `WATCH`, `FANKLE`, `DEID` |
| `LEVEL_COLORS` | ANSI colour codes per level: cyan, green, yellow, red, bold magenta |
| `RESET` | ANSI reset code `\x1b[0m` |

## Functions

### get_logger()
Returns the global `Logger` (name `root`, level `LOG_INFO`), creating it on first use. Configure it with its methods, for example `get_logger().set_level(LOG_DEBUG)`.

### log_debug(message)
Logs at `LOG_DEBUG` on the global logger (hidden at the default level).

### log_info(message)
Logs at `LOG_INFO` on the global logger.

### log_warn(message)
Logs at `LOG_WARN` on the global logger.

### log_error(message)
Logs at `LOG_ERROR` on the global logger.

### log_fatal(message)
Logs at `LOG_FATAL` on the global logger.

### log_assert(condition, message)
If `condition` is falsy, logs `ASSERTION FAILED: <message>` at error level on the global logger. It never throws.

### log_assert_eq(actual, expected, message = "Values should be equal")
If the values differ, logs three error lines: `ASSERTION FAILED: <message>`, `  Expected: <expected>` and `  Actual:   <actual>`.

## Classes

### Logger
`Logger(name = "root", level = 1)`. Fields: `name`, `level`, `use_colors` (`aye`), `use_scots` (`nae`), `show_timestamp` (`aye`), `handlers`, `log_count`. The setting methods return the logger so they chain.

- `set_level(level)` sets the minimum level.
- `colors(enabled)` turns ANSI colours on or off.
- `scots_mode(enabled)` switches level names to the Scots set.
- `timestamps(enabled)` turns the `[<ms>]` prefix on or off.
- `add_handler(handler)` adds a function `handler(level, message, formatted)` called for each emitted line.
- `format_message(level, message)` returns the line: optional `[timestamp]`, then `[LEVEL]` (coloured if enabled), then `(name)` unless the name is `root`, then the message.
- `log(level, message)` emits the line with `blether` if `level >= self.level`, adds to `log_count`, and calls the handlers. Returns nothing. A level outside 0-4 fails with an index error.
- `debug(message)`, `info(message)`, `warn(message)`, `error(message)`, `fatal(message)` log at the matching level.
- `havers(message)`, `ken_this(message)`, `watch_yersel(message)`, `fankle(message)`, `deid(message)` are Scots aliases for debug, info, warn, error and fatal.
- `get_count()` returns how many lines have been emitted.
- `child(name)` returns a new `Logger` named `<parent>.<name>` (or just `name` under `root`) with the parent's level, colour, Scots and timestamp settings. Handlers are not inherited.

### LogBuffer
`LogBuffer(max_size = 1000)`. Fields: `entries`, `max_size`. Each entry is `{"level", "message", "formatted", "timestamp"}`. Use it as a handler, for example `logger.add_handler(|lv, m, f| { buf.add(lv, m, f) })`.

- `add(level, message, formatted)` appends an entry and drops the oldest when over `max_size`.
- `get_all()` returns every entry.
- `get_by_level(level)` returns the entries with exactly that level.
- `get_errors()` returns entries at `LOG_ERROR` or above.
- `clear()` empties the buffer.
- `count()` returns the entry count.
- `recent(n = 10)` returns the last `n` entries.

### StructuredLogger
`StructuredLogger(name = "app")` prints one JSON object per line with `json_stringify`. Fields: `name`, `level` (`LOG_INFO`), `context`. It has no `fatal` method.

- `with_context(key, value)` adds a key to every later entry. Returns the logger.
- `clear_context()` removes the context. Returns the logger.
- `set_level(level)` sets the minimum level. Returns the logger.
- `log(level, message, fields = {})` prints `{"timestamp", "level", "logger", "message", ...context, ...fields}` if `level >= self.level`.
- `debug`, `info`, `warn`, `error` each take `(message, fields = {})` and call `log` at that level.

### PerfLogger
`PerfLogger()` times named operations in milliseconds. Fields: `timers`, `measurements`.

- `start(name)` starts a timer called `name`.
- `stop(name)` prints `[PERF] <name>: <n>ms`, stores a measurement `{"name", "duration_ms", "timestamp"}` and returns the duration. For a name that was never started it returns 0 and prints nothing.
- `get_measurements()` returns the stored measurements.
- `average(name)` returns the average duration for that name (integer division for integers), or 0 if there are none.
- `clear()` drops timers and measurements.
- `report()` prints a "Performance Report" with count, average, minimum and maximum per name.

## Example

```mdhavers
fetch "stdlib/logging"

ken log = Logger("app", LOG_DEBUG).colors(nae).timestamps(nae)
ken buf = LogBuffer(10)
log.add_handler(|lv, m, f| { buf.add(lv, m, f) })

log.debug("starting")
log.info("ready")
log.scots_mode(aye)
log.fankle("something broke")

blether log.get_count()
blether buf.count()
blether len(buf.get_errors())
blether log.child("db").format_message(LOG_WARN, "slow query")
```
