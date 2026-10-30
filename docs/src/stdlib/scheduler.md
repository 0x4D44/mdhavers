# scheduler

A simulated, tick-driven task scheduler. Nothing here uses real time: a global counter is advanced by `tick()`, and every "delay", "interval" or "window" is measured in ticks. The module has a `Scheduler` that runs prioritised, delayed and repeating `Task`s, a cron-style expression builder, and small helpers: a rate limiter, a debouncer, a throttler and a retry helper.

Import: `fetch "stdlib/scheduler"`

Pure mdhavers; runs on every backend. Loading the module prints `Scheduler module loaded! Ready tae schedule yer tasks!`. The module defines its own `tick()` (a counter), which shadows the built-in `tick()` once fetched. Helpers starting with `_` (the counter `_tick_counter` and `Scheduler._sort_tasks`) are internal and not documented.

## Constants

| Name | Value |
| --- | --- |
| `PRIORITY_LOW` | 1 |
| `PRIORITY_NORMAL` | 5 (default for new tasks) |
| `PRIORITY_HIGH` | 10 |
| `PRIORITY_URGENT` | 20 |

## Functions

### tick()
Adds 1 to the module's tick counter and returns the new value.

### reset_ticks()
Sets the tick counter back to 0.

### current_tick()
Returns the counter without changing it.

### make_task(name, action)
Returns `Task(name, action)`.

### make_scheduler()
Returns a new `Scheduler`.

### make_cron()
Returns a new `CronSchedule`.

### make_rate_limiter(max_calls, window_size)
Returns `RateLimiter(max_calls, window_size)`.

### make_debouncer(delay)
Returns `Debouncer(delay)`.

### make_throttler(interval)
Returns `Throttler(interval)`.

### make_retry(max_attempts)
Returns `Retry(max_attempts)`.

## Classes

### Task
`Task(name, action)`; `action` is a function called with no arguments. Fields: `name`, `action`, `priority` (5), `delay` (0), `repeat_interval` (0), `max_runs` (1), `run_count`, `enabled`, `last_run` (-1), `next_run` (0), `created_at`. The configuring methods return the task so they chain.

- `with_priority(p)` sets the priority (higher runs first).
- `with_delay(ticks)` sets `next_run` to `current_tick() + ticks`.
- `repeat_every(interval)` repeats every `interval` ticks and sets `max_runs` to -1 (unlimited).
- `run_times(n)` sets the maximum number of runs. Call it after `repeat_every`, because `repeat_every` resets the limit to unlimited.
- `disable()` and `enable()` switch the task off and on.
- `should_run(current_time)` is `aye` when enabled, under its run limit, and `current_time >= next_run`.
- `execute()` calls the action, adds to `run_count`, sets `last_run`, and schedules `next_run` for repeating tasks.
- `is_complete()` is `aye` once `run_count >= max_runs`; always `nae` for unlimited tasks.

### Scheduler
`Scheduler()` has no arguments. Fields: `tasks`, `running`, `current_time`, `completed` (names of finished tasks). Time advances only when you call `step()` or `run()`.

- `add(task)` adds a task. Returns the scheduler.
- `remove(task_name)` removes every task with that name. Returns the scheduler.
- `get_task(name)` returns the task or `naething`.
- `step()` advances the tick, runs every due task (higher priority first, then earlier `next_run`), drops completed tasks, and returns how many tasks ran.
- `run(max_ticks = 100)` calls `step()` until no tasks remain or `max_ticks` steps have run. Returns the number of steps taken. An unlimited repeating task therefore runs until `max_ticks`.
- `stop()` clears the `running` flag. Returns the scheduler.
- `clear()` removes all tasks and the completed list. Returns the scheduler.
- `pending_count()` and `completed_count()` return the list sizes.
- `status()` returns `{"running", "pending", "completed", "current_time"}`.

### CronSchedule
`CronSchedule()` starts as `* * * * *`. It only builds the expression text; it does not run anything. Fields: `minute`, `hour`, `day`, `month`, `weekday` (strings). The builder methods return the object.

- `at_minute(m)`, `at_hour(h)`, `on_day(d)`, `in_month(m)`, `on_weekday(w)` set one field.
- `every_minute()` sets the minute field to `*`.
- `every_hour()` sets minute `0`, hour `*`.
- `daily()` sets minute `0`, hour `0`.
- `weekly()` sets minute `0`, hour `0`, weekday `1`.
- `to_string()` returns `"<minute> <hour> <day> <month> <weekday>"`, for example `30 9 * * 1`.

### RateLimiter
`RateLimiter(max_calls, window_size)` allows `max_calls` calls inside a window of `window_size` ticks.

- `can_proceed()` drops calls older than the window and returns `aye` if fewer than `max_calls` remain.
- `record_call()` records a call at the current tick and returns `aye`, or returns `nae` without recording when the limit is reached.
- `remaining()` returns how many more calls are allowed now.
- `reset()` forgets all calls. Returns the limiter.

### Debouncer
`Debouncer(delay)`; despite the name it behaves as a leading-edge limiter.

- `call(action)` runs `action()` and returns `aye` if at least `delay` ticks have passed since the last run; otherwise it marks the debouncer pending and returns `nae`.
- `force(action)` runs `action()` regardless and returns `aye`.
- `is_pending()` is `aye` if a call was suppressed since the last run.

### Throttler
`Throttler(interval)`.

- `call(action)` runs `action()` and returns `aye` if `interval` ticks have passed since the last run, else returns `nae`.
- `time_until_next()` returns the ticks left before the next call is allowed (0 if allowed now).

### Retry
`Retry(max_attempts)` retries a function that hurls an error. Fields: `max_attempts`, `attempts`, `delay` (1), `backoff_multiplier` (2), `last_error`.

- `with_delay(d)` sets the initial delay in ticks. Returns the object.
- `with_backoff(multiplier)` sets the delay multiplier. Returns the object.
- `execute(action, error_handler = naething)` calls `action()` and returns its result on success. On an error it stores it in `last_error`, calls `error_handler(error, attempt_number)` if given, advances the tick counter by the delay, multiplies the delay, and tries again. After `max_attempts` failures it returns `naething`. The attempt counter persists between calls, so use `reset()` before reusing a `Retry`.
- `reset()` clears attempts, delay and `last_error`. Returns the object.

## Example

```mdhavers
fetch "stdlib/scheduler"

ken log = []
ken s = make_scheduler()
s.add(make_task("first", || shove(log, "first")).with_priority(PRIORITY_HIGH))
s.add(make_task("later", || shove(log, "later")).with_delay(2))
s.add(make_task("ticker", || shove(log, "ticker")).repeat_every(2).run_times(2))

blether s.run(20)
blether log
blether s.status()
blether make_cron().at_minute(30).at_hour(9).on_weekday(1).to_string()
```
