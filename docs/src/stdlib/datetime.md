# datetime

Calendar date and time handling. It provides a `DateTime` class (build one from components or from a Unix timestamp, then format it, name its weekday), a `Duration` class with helper constructors such as `hours(25)`, a "time ago" describer, and a few Scottish date checks (Hogmanay, Burns Night, St Andrew's Day).

Import: `fetch "stdlib/datetime"`

Pure mdhavers on top of `noo()`, so it runs on every backend that supports `noo()`. Loading the module prints `Datetime module loaded! Whit time is it? Time tae code!`. All calendar maths is UTC with the Gregorian rules; no time zones. Helpers starting with `_` are internal and not documented (this module has none).

## Constants

| Name | Value |
| --- | --- |
| `SECONDS_PER_MINUTE`, `SECONDS_PER_HOUR`, `SECONDS_PER_DAY`, `SECONDS_PER_WEEK` | 60, 3600, 86400, 604800 |
| `DAYS_PER_WEEK`, `MONTHS_PER_YEAR` | 7, 12 |
| `DAY_NAMES`, `DAY_NAMES_SHORT`, `DAY_NAMES_SCOTS` | Seven names, Sunday first (index 0) |
| `MONTH_NAMES`, `MONTH_NAMES_SHORT` | Twelve names, January first |
| `DAYS_IN_MONTH` | `[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]` (non-leap) |

## Functions

### timestamp()
Current Unix time in whole seconds.

### timestamp_ms()
Current Unix time in milliseconds (`noo()`).

### is_leap_year(year)
Gregorian leap-year test. Returns `aye` or `nae`.

### days_in_month(year, month)
Days in `month` (1-12) of `year`, with February 29 in leap years. An out-of-range month is an index error from the list lookup.

### day_of_week(year, month, day)
Day of the week using Zeller's congruence. Returns `0` for Sunday through `6` for Saturday. For example `day_of_week(2024, 3, 15)` is `5` (Friday).

### noo_datetime()
A `DateTime` for the current moment (UTC).

### make_datetime(year, month, day, hour = 0, minute = 0, second = 0)
Builds a `DateTime` from components. No range checking is done.

### pluralize(n, word)
Returns `"1 egg"` for 1, otherwise `"2 eggs"` (appends `s`).

### seconds(n)
A `Duration` of `n` seconds.

### minutes(n)
A `Duration` of `n` minutes.

### hours(n)
A `Duration` of `n` hours.

### days(n)
A `Duration` of `n` days.

### weeks(n)
A `Duration` of `n` weeks.

### time_ago(ts)
Describes a Unix-seconds timestamp relative to now in English: `just noo`, `5 minutes ago`, `1 hour ago`, `2 days ago`, `3 weeks ago`, `2 months ago`, `1 year ago`. A timestamp in the future gives `in the future`.

### scots_greeting_fer_time()
`Guid mornin'!` before 12:00, `Guid efternoon!` before 17:00, `Guid evenin'!` before 21:00, otherwise `Guid nicht!` (current UTC hour).

### is_hogmanay(dt)
True when `dt.month == 12` and `dt.day == 31`.

### is_burns_nicht(dt)
True for 25 January.

### is_st_andrews_day(dt)
True for 30 November.

## Classes

### DateTime
`DateTime(year = 1970, month = 1, day = 1, hour = 0, minute = 0, second = 0)`. Fields are the six constructor names.

- `from_timestamp(ts)` sets the fields from Unix seconds (UTC) and returns the same object. Intended for timestamps at or after the epoch; negative values are not handled.
- `weekday()` returns 0 (Sunday) to 6 (Saturday).
- `day_name()` returns the English day name.
- `day_name_scots()` returns the Scots day name (`Soonday`, `Moanday`, `Tysday`, `Wadnesday`, `Fuirsday`, `Fryday`, `Setterday`).
- `month_name()` returns the English month name.
- `is_weekend()` is `aye` for Saturday and Sunday.
- `format(fmt = "%Y-%m-%d %H:%M:%S")` replaces these codes: `%Y` year, `%m` month, `%B` month name, `%b` short month, `%d` day, `%A` day name, `%a` short day, `%H` hour (24h), `%I` hour (12h), `%p` `AM`/`PM`, `%M` minute, `%S` second. Numbers are zero-padded except the year. Other text is left alone.
- `iso()` returns `YYYY-MM-DDTHH:MM:SS`.
- `human()` returns for example `Friday, March 15, 2024 at 02:30 PM`.
- `scots()` returns for example `Fryday, 15 March 2024` (English month name).
- `tae_string()` returns `format()`. Note that `blether dt` prints `<DateTime instance>`; call `dt.iso()` or `dt.tae_string()` to see the date.

### Duration
`Duration(seconds = 0)` stores `total_seconds`.

- `from_parts(days = 0, hours = 0, minutes = 0, seconds = 0)` sets `total_seconds` from the parts and returns the same object.
- `days()` returns whole days.
- `hours()` returns the hour part (0-23).
- `minutes()` returns the minute part (0-59).
- `seconds()` returns the second part (0-59).
- `human()` returns for example `1 day, 2 hours, 3 minutes, 4 seconds`; zero parts are skipped, and zero duration gives `0 seconds`.
- `short()` returns for example `1d 2h 3m 4s`.

## Compared with time and dates

- [time](./time.md) is about clocks: stopwatches, timers, sleeping, and UTC time of day. It has no calendar.
- [dates](./dates.md) represents a date as a plain dict (`{"year":..,"month":..,"day":..}`) and offers date arithmetic (`add_days`, `days_between`) and Scottish holiday names. `datetime` has an object with a time of day and formatting, but no date arithmetic.
- `datetime` and `dates` both define `is_leap_year`, `days_in_month` and `day_of_week`, but with different signatures and numbering: here `day_of_week(year, month, day)` returns 0 for Sunday; in `dates` `day_of_week(date)` returns 0 for Monday. Do not fetch both in one program unless you accept that the last one fetched wins.
- `time` and `datetime` both define `timestamp`, `timestamp_ms` and `time_ago`.

Prefer `datetime` for formatting a point in time, `dates` for pure date arithmetic, and `time` for timing code.

## Example

```mdhavers
fetch "stdlib/datetime"

ken dt = make_datetime(2024, 3, 15, 14, 30, 5)
blether dt.iso()
blether dt.human()
blether dt.scots()
blether dt.format("%a %d %b %Y")
blether dt.is_weekend()

ken stamp = DateTime()
stamp.from_timestamp(1700000000)
blether stamp.iso()

blether hours(25).human()
```
