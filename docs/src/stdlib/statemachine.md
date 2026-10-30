# statemachine

An event-keyed state machine: you give it a starting state and a set of valid states, then add transitions of the form "event takes from-state to to-state". It has enter, exit and transition handlers, history, a fluent builder, DOT export and a few ready-made machines.

Import: `fetch "stdlib/statemachine"`

Pure mdhavers; it runs on every backend. Loading the module prints `State machine module loaded! Ready tae transition!`. The valid-states argument must be a `creel` (set), for example `creel(["idle", "running"])`, not a list or a `{...}` literal; a list makes `add_transition` fail with `is_in_creel() needs a creel (set)`. Defines `StateMachine` and `door_machine`, as [fsm](./fsm.md) does, so do not fetch both. Helpers starting with `_` are internal and not documented (this module has none).

## Functions

### traffic_light()
States `red` (initial), `green`, `yellow`; event `next` cycles red, green, yellow.

### on_off_switch()
States `off` (initial) and `on`; events `toggle`, `turn_on`, `turn_off`.

### door_machine()
States `closed` (initial), `open`, `locked`; events `open`, `close`, `lock`, `unlock`.

### connection_machine()
`disconnected` (initial), `connecting`, `connected`, `error`; events `connect`, `connected`, `disconnect`, `error`, `retry`, `reset`.

### game_state_machine()
`menu` (initial), `loading`, `playing`, `paused`, `game_over`; events `start_game`, `loaded`, `pause`, `resume`, `game_over`, `restart`, `main_menu`.

### document_workflow()
`draft` (initial), `review`, `approved`, `published`, `archived`; events `submit`, `approve`, `reject`, `publish`, `archive`, `restore`.

### state_machine_builder(initial_state)
Returns `StateMachineBuilder(initial_state)`.

### sm_to_dot(sm, name = "StateMachine")
Returns a Graphviz DOT string: a `digraph` with left-to-right layout, the initial state as a double circle and one labelled edge per transition.

### sm_info(sm)
Prints the current state, available events, history and valid states.

### validate_event_sequence(sm, events)
Replays `events` on a throw-away machine started from `sm`'s initial state (its first history entry) with `sm`'s transitions. Returns `aye` if every event could fire, `nae` at the first that cannot. The original is untouched, but note that handlers are not copied.

## Classes

### StateMachine
Constructor: `StateMachine(initial_state, valid_states)`. Fields: `current_state`, `valid_states`, `transitions` (`{event: {from_state: to_state}}`), `history` (starts as `[initial_state]`), `on_enter_handlers`, `on_exit_handlers`, `on_transition_handlers`.
- `state()`, `is_in(state)`.
- `add_transition(event, from_state, to_state)` - prints `Warning: '<state>' is nae a valid state` for a state outside `valid_states`, but still adds it. It has no `gie`; in practice it yields the to-state (the last assignment), so do not rely on the result. A later transition for the same event and from-state replaces the earlier one.
- `add_transitions(transitions_list)` - list of `[event, from, to]` triples.
- `trigger(event)` - `aye` if it transitioned, `nae` if the event is unknown or has no transition from the current state. Order: exit handler, state change and history append, transition handlers, enter handler.
- `must_trigger(event)` - despite the name it does not hurl. On failure it prints `StateMachine Error: ...Cannae trigger '<event>' from state '<state>'` and carries on.
- `on_enter(state, handler)`, `on_exit(state, handler)` - one handler per state, called as `handler(state, event)`. `on_transition(handler)` - any number, called as `handler(from, event, to)`.
- `available_events()`, `can_trigger(event)`, `peek_next(event)` (target state or `naething`).
- `get_history()` - visited state names, starting with the initial one.
- `reset()` - back to the first history entry; history cleared to that entry.
- `tae_string()` - `StateMachine(state='...', available=[...])`.

### StateMachineBuilder
Fluent builder. Constructor: `StateMachineBuilder(initial_state)`.
- `add_state(name)`, `add_states(state_list)`, `add_transition(event, from_state, to_state)` - all return `masel`; states named in transitions are added automatically.
- `build()` - returns the configured `StateMachine`.

### HierarchicalState
A plain record for parent and child state relationships. It is not wired into `StateMachine`; it does not change how events are handled. Constructor: `HierarchicalState(name, parent = naething)`. Fields: `name`, `parent`, `children`, `initial_child`.
- `add_child(child_name)` - the first child becomes `initial_child`. `set_initial_child(child_name)`. `is_parent()`, `has_parent()`.

## Example

```mdhavers
fetch "stdlib/statemachine"

dae entered(state, event) {
    blether f"entered {state} via {event}"
}

ken sm = state_machine_builder("idle").add_transition("start", "idle", "running").add_transition("stop", "running", "idle").build()
sm.on_enter("running", entered)
blether sm.trigger("start")
blether sm.state()
blether sm.available_events()
blether sm.can_trigger("start")
```

## Compared with fsm

See the table in [fsm](./fsm.md). In short: this module keys transitions by event, takes the valid states as a `creel`, only warns about unknown states, has no guards or final states, and adds a builder, DOT export and sequence validation. Use [fsm](./fsm.md) for guards and final states.
