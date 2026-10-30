# events

A small publish/subscribe toolkit: `EventEmitter` (with one-time listeners and history), a lazily created global event bus, a deferred `EventQueue`, a reactive `Signal`, an `EventCounter`, an `EventLogger`, and constants for common game, UI and system event names.

Import: `fetch "stdlib/events"`

Pure mdhavers; it runs on every backend. Loading the module prints `Events module loaded! Ready tae notify!`. Handlers receive an event dict `{"name", "data", "timestamp"}` (timestamp from `noo()`). Handlers are lambdas or `dae` functions; a brace-less lambda (`|e| ...`) is a single expression, so use a `{ }` block lambda or a named `dae` for anything with statements. Helpers starting with `_` are internal and not documented (`_global_bus` is the bus singleton).

## Constants

Event-name strings: `GAME_START`, `GAME_PAUSE`, `GAME_RESUME`, `GAME_OVER` (`"game:start"` and so on), `PLAYER_MOVE`, `PLAYER_JUMP`, `PLAYER_ATTACK`, `PLAYER_DAMAGE`, `PLAYER_DEATH`, `ENEMY_SPAWN`, `ENEMY_DEATH`, `ITEM_PICKUP`, `LEVEL_START`, `LEVEL_COMPLETE`, `BUTTON_CLICK` (`"ui:button:click"`), `FORM_SUBMIT`, `INPUT_CHANGE`, `MENU_OPEN`, `MENU_CLOSE`, `APP_START`, `APP_STOP`, `ERROR` (`"system:error"`), `WARNING` (`"system:warning"`).

## Functions

### get_event_bus()
Returns the global `EventEmitter`, creating it on first use.

### subscribe(event_name, handler)
`get_event_bus().on(event_name, handler)`. Returns the bus.

### unsubscribe(event_name)
Removes all listeners for that event on the global bus (there is no per-handler removal). Returns the bus.

### publish(event_name, data = naething)
Emits on the global bus. Returns `aye` if at least one listener ran.

### create_logged_emitter(prefix = "[EVENT]")
Returns `{"emitter": EventEmitter(), "logger": EventLogger(prefix)}`. The logger is not attached to the emitter; wire it up with `emitter.on(name, logger.handler())`.

## Classes

### EventEmitter
Constructor: `EventEmitter()`. Fields: `listeners`, `once_listeners`, `event_history`, `max_history` (100).
- `on(event_name, handler)` - add a listener, returns `masel`.
- `once(event_name, handler)` - runs on the next emit only, returns `masel`.
- `off_all(event_name)` - remove every listener for the event (regular and once), returns `masel`.
- `emit(event_name, data = naething)` - records history, calls regular listeners then once listeners in registration order, clears the once list. Returns `aye` if any handler ran, else `nae`. A handler that hurls aborts the emit.
- `has_listeners(event_name)`, `listener_count(event_name)`, `event_names()` (names that have listeners).
- `history(count = 10)` - the last `count` events.
- `clear()` - remove all listeners. `clear_history()`.

### EventQueue
Events held for later processing. Constructor: `EventQueue()`. Fields: `queue`, `processing`.
- `enqueue(event_name, data = naething)`.
- `process_one(handler)` - calls `handler(name, data)` for the oldest event; `aye` if there was one.
- `process_all(handler)` - drains the queue (including events enqueued meanwhile), returns the count handled.
- `length()`, `clear()`, `is_processing()`, `peek()` (next event dict `{"name", "data"}` or `naething`).

### Signal
A value that notifies subscribers when set. Constructor: `Signal(initial_value = naething)`. Fields: `value`, `subscribers`.
- `get()`, `set(new_value)` (calls `subscriber(new_value, old_value)` for each; returns `naething`), `subscribe(handler)` (returns `masel`), `update(fn)` (sets `fn(value)`), `subscriber_count()`.
- `set` notifies even if the value did not change.

### EventCounter
Constructor: `EventCounter()`. Field: `counts`.
- `count(event_name)` - increment. `get_count(event_name)` - count or 0. `get_all()` - the dict. `reset()`. `total()`.

### EventLogger
Constructor: `EventLogger(prefix = "[EVENT]")`. Fields: `prefix`, `logs`, `enabled`.
- `log(event)` - when enabled, builds `"<prefix> <name>: <data>"`, stores it and prints it with `blether`.
- `handler()` - returns a function that logs the event it is given; pass it to `on`.
- `get_logs()`, `clear()`, `enable()`, `disable()`.

## Example

```mdhavers
fetch "stdlib/events"

dae on_hit(e) {
    blether f"hit for {e['data']}"
}

ken em = EventEmitter()
em.on(PLAYER_DAMAGE, on_hit)
blether em.emit(PLAYER_DAMAGE, 5)
blether em.emit("nobody:listening")

ken score = Signal(0)
score.subscribe(|n, o| n)
score.set(10)
blether score.get()
```

## Compared with store

[store](./store.md) also has subscriptions (`Reactive`, `Store.subscribe`). Use events for named, fire-and-forget notifications; use store when you want a single state tree changed through reducers.
