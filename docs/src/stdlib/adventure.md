# adventure

A text-adventure engine: items, non-player characters, rooms with exits, locks and darkness, a player with inventory and flags, and an `AdventureGame` that parses typed commands (`take lamp`, `north`, `give coin to piper` and so on) and runs a read-eval-print loop.

Import: `fetch "stdlib/adventure"`

Pure mdhavers, but `AdventureGame.run()` reads from standard input with `speir`, so it is interactive and needs a terminal or piped input. Everything else can be driven from code with `process_command`. Loading the module prints `Adventure module loaded! Ready tae create interactive fiction!`. Helpers starting with `_` are internal and not documented.

Setters on `Item`, `NPC`, `Room`, `Player` and `AdventureGame` return `masel`, so calls chain.

## Functions

### adventure_speir_or_awa(prompt)
Asks for a line with `speir(prompt)`. If input has ended (`speir` gives `naething`) it prints `Input's ended - cheerio, and haste ye back!` and returns `naething`; otherwise it returns the line. `run()` uses it.

### make_item(id, name, description)
Returns `Item(id, name, description)`.

### make_npc(id, name, description)
Returns `NPC(id, name, description)`.

### make_room(id, name, description)
Returns `Room(id, name, description)`.

### make_adventure(title)
Returns `AdventureGame(title)`.

## Constants

- Directions: `NORTH`, `SOUTH`, `EAST`, `WEST`, `UP`, `DOWN`, `IN`, `OUT` (the strings `"north"`, `"south"` and so on).
- `DIRECTION_OPPOSITES`: dictionary mapping each direction to its opposite. The engine does not use it; it is there for your own room-building code.
- `DIRECTION_ALIASES`: `n`, `s`, `e`, `w`, `u`, `d` to the full direction names.

## Classes

### Item
`Item(id, name, description)`. Fields: `id`, `name`, `description`, `takeable` (`aye`), `useable` (`nae`), `use_message`, `examine_text`, `aliases`, `hidden` (`nae`), `properties`.

- `with_aliases(aliases)`: list of extra names the player can type.
- `not_takeable()`: make it impossible to pick up.
- `useable_with(message)`: make it usable; `message` is what `use` prints.
- `with_examine(text)`: text shown by `examine` instead of the description.
- `set_hidden(hidden)`: hidden items are not listed or found in a room.
- `set_property(key, value)`, `get_property(key)`: free-form data; `get_property` gives `naething` if missing.
- `matches(text)`: `aye` if `text` equals the name, id or an alias, ignoring case.

### NPC
`NPC(id, name, description)`. Fields: `dialogue` (topic to text), `current_topic` (`"greeting"`), `talked_to`, `friendly`, `gives_item`, `wants_item`, `quest_complete`.

- `add_dialogue(topic, text)`: topic keys should be lower case, because `ask_about` lower-cases the topic. The topic `"greeting"` is what `talk` says first and `"thanks"` is used after a completed quest.
- `set_quest(wants, gives)`: the item id the NPC wants and the id it gives back.
- `talk()`: the text for the current topic, or `<name> has naething tae say.`
- `ask_about(topic)`: the text for that topic, or `<name> doesnae ken aboot that.`
- `give_item(item_id)`: returns `{"accepted": aye, "gives": id}` when the id matches the wanted item and the quest is not already done (the topic then becomes `"thanks"`), otherwise `{"accepted": nae, "message": "<name> doesnae want that."}`.

### Room
`Room(id, name, description)`. Fields: `exits` (direction to room id), `items`, `npcs`, `visited`, `first_visit_text`, `dark`, `locked_exits`, `on_enter_callback`.

- `add_exit(direction, room_id)`: one-way exit; add the return exit yourself.
- `lock_exit(direction, key_id)`: the exit needs the item with that id in the inventory.
- `unlock_exit(direction)`: remove the lock.
- `add_item(item)`, `remove_item(item_id)` (returns the removed item or `naething`), `get_item(text)` (a visible item matching `text`, or `naething`).
- `add_npc(npc)`, `get_npc(text)` (match on name or id, ignoring case).
- `with_first_visit(text)`: text prepended on the first description.
- `set_dark(dark)`: a dark room describes as `It's pitch black. Ye cannae see a thing!`
- `on_enter(callback)`: a function called as `callback(game)` when the player moves in.
- `describe()`: the description, with first-visit text (if not yet visited), `Exits: ...`, `Ye can see: ...` for visible items, and `... is here.` for NPCs.

### Player
`Player()`. Fields: `inventory`, `max_inventory` (10), `health` (100), `score`, `flags`.

- `add_item(item)`: returns `{"success": aye}`, or `{"success": nae, "message": "Yer pockets are full!"}` at the limit.
- `remove_item(item_id)`: returns the removed item or `naething`.
- `has_item(item_id)`: by id.
- `get_item(text)`: by name, id or alias, or `naething`.
- `list_inventory()`: `Ye're carrying: a, b` or `Ye're carrying naething.`
- `set_flag(flag, value = aye)`, `get_flag(flag)` (`nae` if unset).
- `add_score(points)`.

### AdventureGame
`AdventureGame(title)`. Fields: `title`, `rooms`, `current_room_id`, `player`, `running`, `commands`, `intro_text`, `win_condition`, `turns`.

- `add_room(room)`, `set_start_room(room_id)`, `set_intro(text)`.
- `set_win_condition(callback)`: a function called as `callback(game)` returning true when the player has won. See the notes on when it is checked.
- `current_room()`: the current `Room`, or `naething`.
- `move(direction)`: accepts aliases. Returns `Ye're lost in the void!`, `Ye cannae go that way.`, `That way is locked.`, or the new room's description. If the exit is locked and the player has the key it unlocks the exit and returns `Ye unlock the way <dir> and proceed.` without moving; move again to go through.
- `take(item_name)`, `drop(item_name)`, `examine(target)`, `talk_to(npc_name)`, `ask_about(npc_name, topic)`, `give_to(item_name, npc_name)`, `use_item(item_name)`: each returns the message string to print, for example `There's nae <x> here.`, `Ye cannae take the <x>.`, `Ye dinnae have a <x>.`. `give_to` does not actually hand a reward item to the player, it only prints a message.
- `process_command(input)`: parses one line and returns the response string. Commands: directions and `n s e w u d in out`, `look`/`l`, `inventory`/`inv`/`i`, `take`/`get`, `drop`, `examine`/`x`, `talk`/`say`, `use`, `give <item> to <person>`, `score`, `help`, `quit`/`q`. `ask` is registered in `commands` but `process_command` has no handler for it (checked: `ask piper tune` gives the "dinnae understand" reply), so call `ask_about(npc_name, topic)` from code. Anything else gives `Ah dinnae understand that. Try 'help' fer commands.` Each call counts as a turn.
- `start()`: prints the intro and the first room's description.
- `run()`: `start()`, then loops reading commands with `> ` until `quit`, a win, or the end of input, and prints `Final Score: <n>  Turns: <n>`.

## Example

```mdhavers
fetch "stdlib/adventure"

ken g = make_adventure("Wee Hoose")
ken hall = make_room("hall", "Hall", "A draughty hall.")
ken yard = make_room("yard", "Yard", "A muddy yard.")
hall.add_exit("north", "yard")
yard.add_exit("south", "hall")
hall.add_item(make_item("key", "rusty key", "A rusty key.").with_aliases(["key"]))
g.add_room(hall).add_room(yard).set_start_room("hall")
g.start()
blether g.process_command("take key")
blether g.process_command("north")
blether g.process_command("inventory")
```

## Notes on current behaviour

- The win condition is only tested when a command is not recognised, so a normal command such as `take` or `north` never triggers it.
- `first_visit_text` is only shown by `start()`; `move()` marks the room visited before describing it.
