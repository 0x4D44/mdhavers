# event_loop

Thin wrappers over the native event-loop built-ins (`event_loop_new`, `event_watch_read`, `timer_after` and friends) for socket readiness watchers and timers, plus `event_loop_run`, a loop that polls and dispatches callbacks until a stop event arrives.

Import: `fetch "stdlib/event_loop"`

Backend: needs the native event-loop built-ins (sockets and timers). It is available in the interpreter and the native (LLVM) backend; it is not meant for the JavaScript or WASM compilers. Loading the module prints nothing. Helpers starting with `_` are internal and not documented (this module has none).

Poll results are lists of event dicts. A timer event looks like `{"kind": "timer", "id": 1, "callback": <function>}`; a stop event is `{"kind": "stop"}`.

## Functions

### make_event_loop()
Creates a loop; returns `event_loop_new()`.

### watch_read(loop, sock, callback)
Calls the callback when `sock` is readable; returns `event_watch_read(...)`.

### watch_write(loop, sock, callback)
Same, for writability.

### unwatch(loop, sock)
Stops watching `sock`; returns `aye` if a watcher was removed, else `nae`.

### timer_after_ms(loop, ms, callback)
Fires once after `ms` milliseconds. Returns the timer id.

### timer_every_ms(loop, ms, callback)
Fires repeatedly every `ms` milliseconds. Returns the timer id.

### timer_cancel_o(loop, timer_id)
Cancels a timer; returns `aye` when it existed, `nae` otherwise (including a one-shot timer that has already fired).

### poll_loop(loop, timeout_ms = -1)
Waits up to `timeout_ms` (negative waits indefinitely) and returns the list of ready event dicts. Does not call the callbacks itself.

### stop_event_loop(loop)
Marks the loop stopped and returns `naething`: the next poll yields `{"kind": "stop"}`. The flag is sticky, so every later poll on that loop yields a stop event again (checked: timers added afterwards do not fire).

### event_loop_run(loop, timeout_ms = -1)
Repeats `poll_loop`; for each event, ends the loop on a `"stop"` kind, else calls `event["callback"](event)` if the event has a callback. Returns `naething`. Call `stop_event_loop(loop)` from a callback to leave the loop.

## Example

```mdhavers
fetch "stdlib/event_loop"

ken lp = make_event_loop()
ken count = 0

dae tick(ev) {
    count = count + 1
    blether f"tick {count}"
    gin count == 3 {
        stop_event_loop(lp)
    }
}

timer_every_ms(lp, 5, tick)
event_loop_run(lp, 50)
blether "done"
```

## Compared with concurrency

[concurrency](./concurrency.md) is for threads, locks and channels; this module is single-threaded readiness and timer dispatch. Use the event loop for I/O and timers on one thread. [scheduler](./scheduler.md) is a separate timer module.
