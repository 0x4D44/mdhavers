# dates

Date utilities that treat a date as a plain dict `{"year": y, "month": m, "day": d}`. It can parse and format dates (ISO, UK and Scots styles), compare them, add days, months and years, count the days between two dates, and name Scottish holidays such as Hogmanay and Burns Night.

Import: `fetch "stdlib/dates"`

Pure mdhavers; runs on every backend. Loading the module prints `Dates module loaded! Time tae get crackin'!`. Dates are not validated: `mak_date(2024, 13, 45)` is accepted. Helpers starting with `_` are internal and not documented (this module has none).

## Constants

### SCOTS_DAYS
Scots day names, Monday first: `Monanday`, `Tuisday`, `Wadensday`, `Fuirsday`, `Fryday`, `Setterday`, `Sabbath` (Sunday). Index matches the result of `day_of_week`.

### SCOTS_MONTHS
Scots month names, January first (`Januar`, `Februar`, `Mairch`, `Aprile`, `Mey`, `Juin`, `Julie`, ...).

## Functions

### mak_date(year, month, day)
Returns the dict `{"year": year, "month": month, "day": day}`.

### mak_datetime(year, month, day, hour = 0, minute = 0, second = 0)
Returns a dict with those six keys.

### parse_date(s)
Parses `"2024-03-15"` (year first, split on `-`) or `"15/03/2024"` (day first, split on `/`). Returns a date dict, or `naething` if the string does not have three parts. A non-numeric part raises the `tae_int` error, for example `Cannae turn 'ab' intae an integer`.

### format_iso(date)
`"2024-03-15"`.

### format_uk(date)
`"15/03/2024"`.

### format_scots(date)
`"15 Mairch 2024"`.

### pad_num(n, width)
Left-pads the string form of `n` with zeros to `width`: `pad_num(5, 3)` is `"005"`.

### is_leap_year(year)
Gregorian leap-year test.

### days_in_month(year, month)
Days in the month (1-12), 29 for February in leap years.

### day_of_week(date)
Zeller's congruence on a date dict. Returns `0` for Monday up to `6` for Sunday (Monday-first, unlike [datetime](./datetime.md)). `2024-03-15` gives `4` (Friday).

### scots_day_name(date)
`SCOTS_DAYS[day_of_week(date)]`.

### scots_month_name(date)
`SCOTS_MONTHS[date["month"] - 1]`.

### compare_dates(a, b)
Returns `-1`, `0` or `1` by year, then month, then day.

### same_date(a, b)
True when `compare_dates(a, b) == 0`.

### is_before(a, b)
True when `a` is earlier than `b`.

### is_after(a, b)
True when `a` is later than `b`.

### add_days(date, days)
Returns a new date `days` later (negative values go back). Works one month at a time, so very large offsets are slow.

### add_months(date, months)
Returns a new date; the day is clamped to the length of the target month (31 Jan plus one month is 29 Feb 2024).

### add_years(date, years)
Returns a new date; 29 February in a non-leap target year becomes 28 February.

### days_between(a, b)
Whole days from `a` to `b`; negative when `a` is after `b`. Counts one day at a time, so it is slow for dates many years apart.

### st_andrews_day(year)
Returns 30 November of `year`.

### burns_night(year)
Returns 25 January.

### hogmanay(year)
Returns 31 December.

### new_years_day(year)
Returns 1 January.

### is_scots_holiday(date)
True for 1 and 2 January, 25 January, 30 November and 31 December.

### scots_holiday_name(date)
Returns `Ne'erday`, `Second Ne'erday`, `Burns Night`, `St Andrew's Day` or `Hogmanay`; otherwise `naething`.

### format_duration(days)
Describes a number of days as a relative phrase: `the noo` (0), `the morra` (1), `yestreen` (-1), `3 days syne`, `2 weeks syne`, `in 4 days`, `in 2 weeks`, `in 3 months`, `in 2 years`. Weeks, months and years are approximate (7, 30 and 365 days). Note that this takes days, not seconds, unlike `format_duration` in [time](./time.md).

## Compared with datetime

[datetime](./datetime.md) gives a `DateTime` object with a time of day, formatting codes and a `Duration` class, but no date arithmetic. `dates` gives dict-based dates with comparison and arithmetic, and Scottish holiday lookups. Both define `is_leap_year`, `days_in_month` and `day_of_week`, with different argument shapes and weekday numbering (here Monday is 0), and the Scots day-name spellings differ between the modules. [time](./time.md) also defines `format_duration`, with a different meaning. Pick one module per program. Prefer `dates` for adding to or comparing calendar dates, and `datetime` for formatting a timestamp.

## Example

```mdhavers
fetch "stdlib/dates"

ken d = parse_date("2024-03-15")
blether format_scots(d)
blether scots_day_name(d)
blether format_iso(add_days(d, 20))
blether days_between(d, hogmanay(2024))
blether scots_holiday_name(new_years_day(2025))
```
