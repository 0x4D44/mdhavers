# fsm

A finite state machine with guards, enter, exit and transition callbacks, history and a text display, plus ready-made machines (toggle, traffic light, door, order workflow, character states, whisky production, ceilidh dance).

Import: `fetch "stdlib/fsm"`

Pure mdhavers; it runs on every backend. Loading the module prints `FSM module loaded! Keep track o' yer states!`. It defines a class called `StateMachine` and a function `door_machine`, exactly as [statemachine](./statemachine.md) does, so do not fetch both in one program. Helpers starting with `_` are internal and not documented (this module has none).

## Functions

### state_machine(name = "Unnamed Machine")
Returns `StateMachine(name)`.

### toggle_machine(name = "Toggle")
States `off` (initial) and `on`; trigger `toggle` flips between them.

### traffic_light_machine()
States `red` (initial), `green`, `yellow`; triggers `go` (red to green), `slow` (green to yellow), `stop` (yellow to red).

### door_machine()
States `closed` (initial), `open`, `locked`; triggers `open`, `lock`, `close`, `unlock`.

### order_workflow_machine()
States `pending` (initial), `confirmed`, `shipped`, `delivered` (final), `cancelled` (final); triggers `confirm`, `cancel`, `ship`, `deliver`.

### character_state_machine()
States `idle` (initial), `walking`, `running`, `jumping`, `falling`, `dead` (final); triggers `walk`, `run`, `jump`, `stop`, `land`, `fall`, `die`.

### whisky_production_machine()
Linear: `malting` (initial) to `mashing` (`grind`), `fermenting` (`add_yeast`), `distilling` (`heat`), `maturing` (`barrel`), `bottling` (`bottle`, final).

### ceilidh_machine()
`ready` (initial), `bow` (`start`), `circle_left`, `circle_right`, `do_si_do`, `promenade` (each by `next`), `finish` (`end`, final).

## Classes

### StateMachine
Constructor: `StateMachine(name = "Unnamed Machine")`. Fields: `name`, `states`, `current`, `initial`, `history`. The machine has no current state until a state is added with `is_initial` set to `aye`.
- `add_state(name, is_initial = nae, is_final = nae)` - returns `masel`. The initial flag also makes it current.
- `add_transition(from_state, trigger, to_state, guard = naething)` - returns `masel`. `guard` is a zero-argument function; the transition is refused when it returns falsy. Both states must already exist, otherwise it fails with `State '<name>' doesnae exist!` (raised through `mak_siccar`).
- `set_on_enter(state, callback)`, `set_on_exit(state, callback)` - zero-argument callbacks. `set_on_transition(callback)` - `callback(from, trigger, to)`. Each returns `masel`.
- `trigger(event)` - returns `aye` if a transition happened, `nae` if the event is unknown from the current state or a guard refused. Order: guard, exit callback, history entry, state change, transition callback, enter callback. With no current state it fails with `State machine hasnae started yet!`.
- `process(events)` - triggers each in turn, returns the list of results.
- `is_in(state)`, `is_done()` (current state is final), `get_state()`, `available_triggers()`, `get_history()` (list of `{"from", "trigger", "to"}`).
- `reset()` - back to the initial state, history cleared, returns `masel`.
- `display()` - prints the machine, its states and transitions.

## Example

```mdhavers
fetch "stdlib/fsm"

dae greet() {
    blether "now on"
}

ken m = toggle_machine()
m.set_on_enter("on", greet)
blether m.trigger("toggle")
blether m.get_state()
blether m.available_triggers()
blether m.trigger("bogus")
```

## Compared with statemachine

[statemachine](./statemachine.md) is the same idea with a different shape.

| | fsm | statemachine |
|---|---|---|
| Setup | `add_state(...)` for every state, then `add_transition(from, trigger, to, guard)` | `StateMachine(initial, valid_states_creel)` then `add_transition(event, from, to)` |
| Argument order | from, trigger, to | event, from, to |
| Unknown state | hard error | prints a warning, still adds it |
| Guards, final states | yes | no |
| Callbacks | zero-argument enter/exit | `(state, event)` enter/exit, multiple transition handlers |
| History | list of `{from, trigger, to}` | list of visited state names |
| Extras | `display()`, `process()`, `is_done()` | builder, DOT export, `can_trigger`, `peek_next`, event-sequence validation |

Prefer fsm when you need guards or final states. Prefer statemachine for the builder, DOT export and simple event-keyed tables. Both modules define `StateMachine` and `door_machine`; fetch only one.
