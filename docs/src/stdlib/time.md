# time

Clock helpers, unit conversion, duration formatting, a `Stopwatch`, a countdown `Timer`, time-of-day checks with Scots greetings, and simple timing and benchmarking functions. It is built on the built-ins `noo()` (milliseconds since the epoch) and `snooze(ms)`.

Import: `fetch "stdlib/time"`

Loading the module prints `Time module loaded! Time an' tide wait fer nae man!`. It works in the interpreter. Time-of-day functions use UTC (the epoch timestamp modulo 86400), not local time. The module defines `benchmark` and `time_it`, which clash with the [benchmark](./benchmark.md) module, and `timestamp`, `timestamp_ms` and `time_ago`, which clash with [datetime](./datetime.md); fetch only one of them, or the last one fetched wins.

Helpers starting with `_` are internal and not documented (this module has none).

Integer arguments to the conversion functions use integer division, so `seconds_tae_minutes(90)` gives `1`. Pass a float (`90.0`) to get `1.5`.

## Constants

| Name | Value |
| --- | --- |
| `MILLISECONDS_PER_SECOND` | 1000 |
| `SECONDS_PER_MINUTE`, `MINUTES_PER_HOUR`, `HOURS_PER_DAY`, `DAYS_PER_WEEK`, `MONTHS_PER_YEAR` | 60, 60, 24, 7, 12 |
| `SECONDS_PER_HOUR`, `SECONDS_PER_DAY`, `SECONDS_PER_WEEK` | 3600, 86400, 604800 |
| `MILLISECONDS_PER_MINUTE`, `MILLISECONDS_PER_HOUR`, `MILLISECONDS_PER_DAY` | 60000, 3600000, 86400000 |
| `DAYS_SCOTS`, `DAYS_ENGLISH` | Seven day names, Sunday first (index 0) |
| `MONTHS_SCOTS`, `MONTHS_ENGLISH` | Twelve month names, January first |

## Functions

### timestamp()
Current time in whole seconds since the epoch (`floor(noo() / 1000)`).

### timestamp_ms()
Current time in milliseconds (alias for `noo()`).

### secs_noo()
Same as `timestamp()`; used internally by the time-of-day helpers.

### wait(seconds)
Sleeps for `seconds` (calls `snooze(seconds * 1000)`). Blocking.

### wait_ms(ms)
Sleeps for `ms` milliseconds.

### seconds_tae_minutes(secs)
`secs / 60`.

### minutes_tae_seconds(mins)
`mins * 60`.

### seconds_tae_hours(secs)
`secs / 3600`.

### hours_tae_seconds(hrs)
`hrs * 3600`.

### seconds_tae_days(secs)
`secs / 86400`.

### days_tae_seconds(days)
`days * 86400`.

### format_duration(total_secs)
Formats seconds as English text, for example `1 day, 1 hour, 1 minute, 1 second`. Zero units are left out; zero seconds gives `0 seconds`. Fractions are floored.

### format_duration_scots(total_secs)
Like `format_duration` but with Scots units: `an oor`, `3 oors`, `a meenit`, `5 meenits`, `a second`. Days stay as `day`/`days`.

### format_duration_compact(total_secs)
Compact form such as `1d 1h 1m 1s`.

### stopwatch()
Returns a new [Stopwatch](#stopwatch).

### timer(seconds)
Returns a new [Timer](#timer) counting down from `seconds`.

### get_hour(timestamp_secs)
Hour (0-23, UTC) of an epoch-seconds timestamp.

### get_minute(timestamp_secs)
Minute (0-59) of a timestamp.

### get_second(timestamp_secs)
Second (0-59) of a timestamp.

### current_hour()
`get_hour` of the current time.

### current_minute()
`get_minute` of the current time.

### current_second()
`get_second` of the current time.

### is_mornin()
True from 06:00 to 11:59.

### is_efternoon()
True from 12:00 to 17:59.

### is_evenin()
True from 18:00 to 21:59.

### is_nicht()
True from 22:00 to 05:59.

### is_wee_small_hours()
True from 00:00 to 05:59.

### time_o_day()
Returns one of `the wee small hours`, `the mornin'`, `high noon` (hour 12), `the efternoon`, `the evenin'`, `late nicht`.

### time_greetin()
A Scots greeting for the current hour, for example `Guid mornin' tae ye!` or `Time fer yer denner!` (hour 12).

### format_12_hour(timestamp_secs)
Formats the time of day as `1:05 in the efternoon` (minutes zero-padded, midnight shows as 12).

### format_24_hour(timestamp_secs)
Formats as `HH:MM:SS`, zero-padded.

### time_it(func)
Calls `func()` once and returns a dict `{"result": <return value>, "ms": <elapsed ms>}`.

### time_an_tell(name, func)
Calls `func()`, prints `  <name>: <n>ms`, and returns the function's result.

### benchmark(name, func, iterations)
Runs `func()` `iterations` times, prints `Benchmark '<name>': <n> runs, average <avg>ms`, and returns the average in milliseconds. The average uses integer division for integer millisecond totals. With `iterations` of 0 it divides by zero.

### countdown(seconds, message = "Starting in")
Prints `<message> <n>...` once per second (blocking, uses `wait(1)`), then `Go!`.

### scottish_countdown(seconds)
Like `countdown` but prints Scots number words (`ten` down to `ane` for values 10 and under; `zero` is never printed) and ends with `AWA' WI' YE!`.

### time_ago(past_timestamp)
Describes an epoch-seconds timestamp relative to now in Scots: `just noo`, `a meenit ago`, `5 meenits ago`, `an oor ago`, `3 oors ago`, `yesterday`, `4 days ago`, `a week ago`, `2 weeks ago`, `a month ago`, `N months ago`. A timestamp in the future also gives `just noo`.

## Classes

### Stopwatch
`Stopwatch()` takes no arguments. Fields: `start_time`, `elapsed` (ms), `running`. Methods that change state return the stopwatch so calls can be chained.

- `start()` starts timing if not running. Returns the stopwatch.
- `stop()` adds the time since `start()` to `elapsed` and stops. Returns the stopwatch.
- `reset()` zeroes the stopwatch and stops it. Returns the stopwatch.
- `restart()` zeroes `elapsed` and starts again. Returns the stopwatch.
- `elapsed_ms()` returns the elapsed milliseconds, including the current run.
- `elapsed_seconds()` returns `elapsed_ms() / 1000.0`.
- `is_running()` returns `aye` or `nae`.
- `tae_string()` returns the elapsed time through `format_duration`.

### Timer
`Timer(duration_secs)` makes a countdown timer. Fields: `duration` (ms), `start_time`, `running`.

- `start()` starts the countdown if not started. Returns the timer.
- `remaining_ms()` returns the full duration while not started, otherwise the time left, never below 0.
- `remaining_seconds()` returns `remaining_ms() / 1000.0`.
- `is_finished()` returns `aye` when `remaining_ms() <= 0`.
- `is_running()` returns `aye` while started and not yet finished.
- `reset()` stops the timer and restores the full duration. Returns the timer.
- `tae_string()` returns the remaining whole seconds through `format_duration`.

## Compared with datetime

[datetime](./datetime.md) works with calendar dates (`DateTime`, `Duration` classes, formatting with `%Y-%m-%d` style codes), while `time` works with clocks, stopwatches and time of day. Use `time` for timing code, pausing, countdowns and "time of day" greetings; use `datetime` or [dates](./dates.md) for calendar dates. Both define `timestamp`, `timestamp_ms` and `time_ago`; `time_ago` here uses Scots wording (`meenits`, `oors`) and treats future times as `just noo`, whereas the datetime version uses English and says `in the future`.

## Example

```mdhavers
fetch "stdlib/time"

blether format_duration(3725)
blether format_duration_compact(90061)
blether format_24_hour(3661)

ken sw = stopwatch()
sw.start()
sw.stop()
blether sw.is_running()
```
