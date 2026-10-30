# System, Files, Time and Logging Built-ins

These built-ins talk to the operating system: the working directory, environment variables, shell commands, files and directories, clocks and dates, and the logging system. They are registered by the interpreter in `src/interpreter.rs` (`Interpreter::define_natives`).

Unless a row says otherwise, a wrong argument type gives an error such as `read_bytes() needs a file path string`, and a failed operating-system call gives a Scots message that quotes the path and the system error.

**Backends.** The interpreter has every built-in on this page. The program in the Examples section, plus `set_crash_handling`, `stopwatch`, `log_init`, `log_event`, `chdir` and `exit`, gives the same output when compiled to JavaScript and run in Node, except that `shell` and `shell_status` are not available there: they fail with `shell() isnae available in JS - it's fer the interpreter an' native builds only` (`runtime/js/core_runtime.js`, `NO_JS`). The same checks on an LLVM native build give the same results, with three differences: `log_event` prints its fields as JSON (`{"k": 1}` rather than `k=1`); after `log_set_filter`, `get_log_level` still reports the old level; and `exit` with a float code gives a wrong exit status. `get_key` is untested outside the interpreter. The WAT/WASM compiler passes each built-in call by name to its host (`__mdh_builtinN`); the `wasm_runner` host runs the interpreter's own code, but that is untested here.

## Working directory and process

| Call | Returns | Notes |
|------|---------|-------|
| `cwd()` | string | Current working directory. Error: `Couldnae get current directory: <err>`. |
| `chdir(path)` | `naething` | Changes the process working directory. Error: `Couldnae change tae directory '<path>': <err>`; `chdir() needs a path string`. |
| `exit(code)` | never returns | Ends the whole process with `code`. A float code is truncated (`exit(2.9)` exits with 2); any other non-number counts as 0. |
| `get_key()` | string | Waits for one key press in raw terminal mode. Letters give themselves, Enter gives `"\n"`, Esc gives `"\x1b"`, Backspace gives `"\x08"`, arrows give `"Left"`, `"Right"`, `"Up"`, `"Down"`; any other key gives `""`. Errors: `Cannae enable raw mode: <err>`, `Cannae read key: <err>`, `Cannae disable raw mode: <err>`. Needs a terminal and the `cli` feature. |
| `set_crash_handling(enabled)` | `naething` | Turns crash handling on or off. Error: `set_crash_handling() expects a boolean`. |

## Environment

| Call | Returns | Notes |
|------|---------|-------|
| `env_get(name)` | string or `naething` | `naething` when the variable is not set. Error: `env_get() needs a variable name string`. |
| `env_set(name, value)` | `naething` | A non-string value is converted to text. Error: `env_set() needs a variable name string`. |
| `env_all()` | dict | Every variable, as string keys and string values. |

## Shell commands

| Call | Returns | Notes |
|------|---------|-------|
| `shell(command)` | string | Runs the command and returns its standard output, or its standard error when stdout is empty. Output keeps its trailing newline. Error: `Shell command failed: <err>`; `shell() needs a command string`. |
| `shell_status(command)` | integer | Runs the command and returns its exit code, or `-1` if it was killed by a signal. Its output goes straight to the terminal. Errors: `Shell command failed: <err>`; `shell_status() needs a command string`. |

The shell is `sh -c` (`cmd /C` on Windows) unless the `MDH_SHELL` environment variable names another one.

## Files and directories

| Call | Returns | Notes |
|------|---------|-------|
| `scrieve_append(path, content)` | `naething` | Appends to the file, creating it if needed. A non-string `content` is converted to text. Errors: `Couldnae open '<path>' fer appendin': <err>`, `Couldnae append tae '<path>': <err>`. |
| `read_bytes(path)` | bytes | Reads a whole file without decoding it. Error: `Couldnae read '<path>': <err>`. |
| `write_bytes(path, bytes)` | `naething` | Writes a bytes value, replacing the file. Errors: `write_bytes() needs bytes`, `Couldnae open '<path>' fer writin': <err>`, `Couldnae write tae '<path>': <err>`. |
| `file_delete(path)` | `naething` | Deletes one file. Error: `Couldnae delete '<path>': <err>`. |
| `file_size(path)` | integer | Size in bytes. Error: `Couldnae get file info fer '<path>': <err>`. |
| `list_dir(path)` | list of strings | Entry names only, not full paths, in the order the operating system gives them (do not rely on it). Error: `Couldnae read directory '<path>': <err>`. |
| `make_dir(path)` | `naething` | Creates the directory and any missing parents; no error if it exists. Error: `Couldnae create directory '<path>': <err>`. |
| `path_join(a, b)` | string | Joins two path parts with the platform separator. An absolute `b` replaces `a`. Error: `path_join() needs strings`. |

`scrieve`, `read_file` and `is_dir` are covered in the main [built-ins reference](../builtins.md).

## Time and dates

Timestamps are Unix seconds (integers).

| Call | Returns | Notes |
|------|---------|-------|
| `timestamp_millis()` | integer | Milliseconds since the Unix epoch. |
| `date_now()` | dict | Local time with integer keys `year`, `month`, `day`, `hour`, `minute`, `second` and `weekday` (0 is Monday). |
| `date_format(timestamp, format)` | string | Formats in local time with chrono (`strftime`) codes such as `%Y-%m-%d`. Errors: `date_format() needs a timestamp`, `date_format() needs a format string`, `Invalid timestamp`. |
| `date_parse(text, format)` | integer | Parses `text` with chrono codes, treated as UTC. Error: `Couldnae parse date '<text>': <err>`. |
| `date_add(timestamp, amount, unit)` | integer | Unit is one of `"seconds"`, `"minutes"`, `"hours"`, `"days"`, `"weeks"`. Error: `Unknown time unit: <unit>`. |
| `date_diff(from, to, unit)` | integer | `to - from`, in `"milliseconds"`, `"seconds"`, `"minutes"`, `"hours"`, `"days"` or `"weeks"`, truncated towards zero. Error: `Unknown time unit: <unit>`. |
| `braw_time()` | string | The UTC time of day in words, for example `It's the mornin' (09:15)`. The bands are `It's the wee small hours` (00-05), `It's the mornin'` (06-11), `It's high noon` (12), `It's the efternoon` (13-17), `It's the evenin'` (18-21) and `It's gettin' late` (22-23). |
| `stopwatch(fn)` | string | A placeholder. It does not call `fn`; it returns the text `Use 'noo()' before and after callin' '<name>' tae time it!`. Error: `stopwatch() needs a function`. Time code with `noo()` instead. |

## Logging

Log levels, from quietest to loudest, are `wheesht` (0), `roar` (1), `holler` (2), `blether` (3), `mutter` (4) and `whisper` (5). Wherever a level is asked for, give either the name or the number.

| Call | Returns | Notes |
|------|---------|-------|
| `set_log_level(level)` | `naething` | Error for a bad name: `Invalid log level '<x>'. Use: wheesht, roar, holler, blether, mutter, or whisper`; for a bad number: `Invalid log level <n>. Use 0-5`; otherwise `set_log_level() expects a string or integer`. |
| `get_log_level()` | string | The current level name in lower case. |
| `log_set_filter(spec)` | `naething` | Sets the filter, for example `"holler"`. Also sets the default level. Errors from a bad spec start with `log_set_filter() `. |
| `log_get_filter()` | string | The spec last given to `log_set_filter`. |
| `log_enabled(level, target)` | boolean | `target` is optional. True when a message at `level` would be shown. Error: `log_enabled() expects 1 or 2 arguments`. |
| `log_event(level, message, fields, target)` | `naething` | Emits one record. `fields` (a dict) and `target` (a string) are optional. Error: `log_event() expects 2-4 arguments`. |
| `log_init(config)` | `naething` | `config` is an optional dict. Keys read from the code: `level`, `filter`, `format` (`"text"`, `"json"` or `"compact"`), `color`, `timestamps` and `sinks`. Error: `log_init() expects 0 or 1 arguments`. |
| `log_span(name, level, fields, target)` | span handle | Makes a span; only `name` is required and `level` defaults to `blether`. Errors: `log_span() expects 1-4 arguments`; `log_span() name must be a string`. |
| `log_span_enter(span)` | `naething` | Error: `log_span_enter() expects a log span handle`. |
| `log_span_exit(span)` | `naething` | Error: `log_span_exit() expects a log span handle`. |
| `log_span_current()` | span handle or `naething` | The innermost entered span, if any. |
| `log_span_in(span, fn)` | the result of `fn()` | Enters the span, calls `fn` with no arguments, leaves the span and returns the result. Error: `log_span_in() expects a log span handle`. |

`log_event`, `log_init` and `log_span_in` only work while the interpreter is running; otherwise they report `... is unavailable outside the interpreter`.

## Examples

This program is safe to run: it only touches files in a folder called `docs_demo_dir` under the current directory.

```mdhavers
make_dir("docs_demo_dir/deep")
scrieve("docs_demo_dir/a.txt", "hi")
scrieve_append("docs_demo_dir/a.txt", " there")
blether read_file("docs_demo_dir/a.txt")
blether file_size("docs_demo_dir/a.txt")
blether list_dir("docs_demo_dir/deep")
blether path_join("docs_demo_dir", "a.txt")
write_bytes("docs_demo_dir/b.bin", read_bytes("docs_demo_dir/a.txt"))
blether file_size("docs_demo_dir/b.bin")
file_delete("docs_demo_dir/b.bin")
file_delete("docs_demo_dir/a.txt")

env_set("MDH_DOC_DEMO", "braw")
blether env_get("MDH_DOC_DEMO")
blether env_get("MDH_DOC_NOPE")

blether date_format(0, "%Y")
blether date_parse("2024-01-02 03:04:05", "%Y-%m-%d %H:%M:%S")
blether date_add(1000, 2, "minutes")
blether date_diff(0, 7200, "hours")

set_log_level("mutter")
blether get_log_level()
blether log_enabled("roar")
blether log_enabled("whisper")
```

Commands that change or end the process are shown as text only:

```text
shell("echo hello")     gives "hello\n"
shell_status("exit 3")  gives 3
chdir("/tmp")           the working directory is now /tmp
exit(2)                 ends the program with status 2
```
