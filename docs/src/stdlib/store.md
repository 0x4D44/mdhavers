# store

Redux-style state management: a `Store` with a reducer, subscribers, middleware, optional undo history and selectors; plus `Reactive` values, `Computed` values, an `ObservableList` and a `StatefulStore` for simple allowed-transition tracking.

Import: `fetch "stdlib/store"`

Pure mdhavers; it runs on every backend. Loading the module prints `Store module loaded! Keep yer state in order!`. An action is a dict `{"type", "payload"}` (see `make_action`). A reducer is `reducer(state, action)` and must return the new state. A lambda body without braces is a single expression; use a `{ }` block lambda or a named `dae` for reducers with branches. Classes starting with `_` (`_CombinedReducer`, `_ValidatorMiddleware`, `_CachedSelector`) are internal and not documented; the functions that return them are.

## Functions

### make_action(action_type, payload = naething)
Returns `{"type": action_type, "payload": payload}`.

### create_store(reducer, initial_state = {})
Returns `Store(reducer, initial_state)`.

### combine_reducers(reducers)
`reducers` is a dict of key to reducer. Returns a reducer that runs each one on `state[key]` (or `{}` when missing) and builds a new dict of the results.

### logger_middleware(store, action)
Prints `[Store] Action: <type>` and, when set, `[Store] Payload: <payload>`. Returns `aye`.

### thunk_middleware(store, action)
If `action` is a function, calls `action(store.dispatch, store.get_state)` and returns `nae` so normal dispatch stops. Otherwise returns `aye`.

### make_validator_middleware(validators)
`validators` maps an action type to `validator(payload)`. Returns a middleware that prints `[Store] Invalid payload fer action: <type>` and returns `nae` (blocking the action) when the validator is falsy.

### create_selector(selector_fn)
Returns a one-argument function that caches the last result and recomputes only when the state compares unequal to the previous one.

### create_action_types(prefix, types)
Returns a dict from each name in `types` to `"<prefix>/<name>"`.

### reactive(initial_value)
Returns `Reactive(initial_value)`.

### computed(dependencies, compute_fn)
Returns `Computed(dependencies, compute_fn)`.

### observable_list(initial_items = [])
Returns `ObservableList(initial_items)`.

### stateful_store(states, initial_state)
Returns `StatefulStore(states, initial_state)`.

## Classes

### Store
Constructor: `Store(reducer, initial_state = {})`. Fields: `state`, `reducer`, `subscribers`, `middleware`, `history`, `history_enabled`, `max_history`.
- `get_state()`.
- `dispatch(action)` - runs each middleware as `mw(store, action)`; if any returns falsy the action is dropped and the current state returned. Otherwise saves history (if enabled), applies the reducer, calls `subscriber(state, action)` for each subscriber, and returns the new state.
- `subscribe(callback)`, `unsubscribe(callback)`, `use(middleware)` - return `masel`.
- `enable_history(max_entries = 50)` - start keeping a shallow copy of the state before each dispatch. Returns `masel`.
- `get_history()` - list of `{"state", "action"}`.
- `undo()` - restores the previous state; `aye` on success, `nae` if history is off or empty. Subscribers are not notified.
- `reset(initial_state = {})` - sets the state (to `{}` unless you pass one, not to the original initial state), clears history and notifies subscribers with action type `"@@RESET"`.
- `select(selector)` - `selector(state)`.

### Reactive
A single observable value. Constructor: `Reactive(initial_value)`.
- `get()`, `set(new_value)` and `update(updater)` (both call `subscriber(new, old)` and return `masel`), `subscribe(callback)`, `unsubscribe(callback)`.

### Computed
A derived value over a list of `Reactive` dependencies. Constructor: `Computed(dependencies, compute_fn)`; `compute_fn` receives the list of current dependency values. Recomputed lazily, on `get()` after any dependency changed.
- `get()`.

### ObservableList
List that notifies on change. Constructor: `ObservableList(initial_items = [])` (uses the given list directly). Subscribers are called as `callback(event_type, data, index, items)` where the type is `"add"`, `"remove"`, `"update"` (data is `{"old", "new"}`) or `"clear"` (index `-1`).
- `get_items()`, `get(index)` (`naething` if out of range), `add(item)`, `remove(index)` (by index, ignored if out of range), `update(index, new_value)`, `clear()`, `length()`, `subscribe(callback)`; mutators return `masel`.
- `find(predicate)`, `filter(predicate)`, `map(transform)` - return a first match or plain new lists.

### StatefulStore
Tracks a current state and which transitions are allowed. Constructor: `StatefulStore(states, initial_state)`; `states` is a dict of state name to a list of allowed next states. Fields: `states`, `current`, `data`, `subscribers`.
- `get_current()`, `get_data()`.
- `can_transition(to_state)`.
- `transition(to_state, data = naething)` - if allowed, moves, stores `data` when not `naething`, calls `subscriber(from, to, data)`, returns `aye`; otherwise `nae`.
- `subscribe(callback)` - returns `masel`.

## Example

```mdhavers
fetch "stdlib/store"

dae counter(state, action) {
    gin action["type"] == "inc" {
        gie {"n": state["n"] + action["payload"]}
    }
    gie state
}

ken st = create_store(counter, {"n": 0})
st.enable_history()
st.dispatch(make_action("inc", 2))
st.dispatch(make_action("inc", 3))
blether st.get_state()
st.undo()
blether st.get_state()

ken a = reactive(1)
ken b = reactive(2)
ken total = computed([a, b], |v| v[0] + v[1])
a.set(10)
blether total.get()
```

## Compared with events and fsm

For named notifications see [events](./events.md). `StatefulStore` is a minimal transition table; for guards, callbacks and history use [fsm](./fsm.md) or [statemachine](./statemachine.md).
