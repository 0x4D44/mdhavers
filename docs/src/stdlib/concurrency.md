# concurrency

Thin wrappers over the native threading and synchronisation built-ins: threads, mutexes, condition variables, atomics and channels. Each function simply calls the matching built-in (`thread_spawn`, `mutex_new`, `atomic_load`, `chan_send` and so on); names ending in `_o` avoid clashing with those built-ins.

Import: `fetch "stdlib/concurrency"`

Backend: needs the native concurrency built-ins; interpreter and native (LLVM) only. In the interpreter, `spawn_thread` accepts only native functions and runs them immediately on the calling thread: passing a user `dae` fails with `thread_spawn() only supports native functions in interpreter`. Use the native backend for real threads. Loading the module prints nothing. Helpers starting with `_` are internal and not documented (this module has none).

## Functions

### spawn_thread(fn, args = [])
Starts `fn` with the list `args`; returns a thread handle (an integer in the interpreter).

### join_thread(handle)
Returns the thread's result. A detached thread cannot be joined (`thread_join() cannot join detached thread`).

### detach_thread(handle)
Lets the thread run on, discarding its result.

### make_mutex()
Returns a new mutex.

### lock_mutex(mutex)
Locks the mutex. In the interpreter it never blocks: it just marks the mutex locked (even if already held) and returns `naething`.

### unlock_mutex(mutex)
Unlocks it; returns `naething`.

### try_lock_mutex(mutex)
Returns `aye` if the lock was taken, `nae` if it was already held.

### make_condvar()
Returns a new condition variable.

### cond_wait(condvar, mutex)
Waits for a signal, releasing `mutex` while waiting. In the interpreter it returns `aye` immediately without waiting.

### cond_timed_wait(condvar, mutex, timeout_ms)
Like `cond_wait` but gives up after `timeout_ms` milliseconds (the interpreter returns `aye` immediately).

### cond_signal(condvar)
Wakes one waiter.

### cond_broadcast(condvar)
Wakes all waiters.

### make_atomic(val)
Returns an atomic holding the integer `val`.

### atomic_get(a)
Reads the value.

### atomic_set(a, val)
Stores `val`.

### atomic_add_o(a, delta)
Adds `delta` and returns the new value (observed: adding 2 to 5 returns 7).

### atomic_cas_o(a, expected, desired)
Compare-and-swap: if the value equals `expected`, sets `desired` and returns `aye`; otherwise `nae`.

### make_chan(capacity = 0)
Returns a channel with the given buffer capacity.

### chan_send_o(ch, val)
Sends `val`. Observed: `aye` on success, `nae` on a closed channel.

### chan_recv_o(ch)
Receives the next value, or `naething` once the channel is closed and drained.

### chan_try_recv_o(ch)
Non-blocking receive; `naething` when nothing is waiting.

### chan_close_o(ch)
Closes the channel; buffered values can still be received.

### chan_closed(ch)
`aye` if closed.

## Example

```mdhavers
fetch "stdlib/concurrency"

ken hits = make_atomic(0)
atomic_add_o(hits, 2)
blether atomic_get(hits)
blether atomic_cas_o(hits, 2, 10)
blether atomic_get(hits)

ken ch = make_chan(2)
chan_send_o(ch, "hello")
blether chan_recv_o(ch)
blether chan_try_recv_o(ch)
chan_close_o(ch)
blether chan_closed(ch)

ken mx = make_mutex()
lock_mutex(mx)
blether try_lock_mutex(mx)
unlock_mutex(mx)
```

## Compared with event_loop

See [event_loop](./event_loop.md): single-threaded readiness and timers, versus shared-state threads here.
