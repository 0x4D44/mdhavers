# game

ASCII game-building toolkit: a character-grid screen buffer with drawing primitives, multi-line sprites, entities with velocity and tags, game state (score, level, lives), frame animation, a tiny particle system, and a few HUD helpers.

Import: `fetch "stdlib/game"`

Pure mdhavers; it needs no native built-ins and runs on any backend that supports classes. Loading the module prints `Game module loaded! Ready tae build braw games!`. The module has no game loop or input handling of its own: you drive `update`/`draw`/`display` yourself. It defines top-level names `BOX_SINGLE`, `BOX_DOUBLE` and `draw_box`, which clash with the same names in [ascii_art](./ascii_art.md); see below.

Methods that change an object return `masel` (the object itself), so calls chain. Helpers starting with `_` are internal and not documented.

## Functions

### make_screen(width, height)
Returns `Screen(width, height)`.

### make_sprite(art)
Returns `Sprite(art)`.

### make_entity(x, y)
Returns `Entity(x, y)`.

### make_game_state()
Returns `GameState()`.

### make_animation(frames)
Returns `Animation(frames)`.

### make_particle_system()
Returns `ParticleSystem()`.

### draw_health_bar(screen, x, y, current, max, width)
Draws `[###---]` at (x, y): `width` cells, of which `(current * width) / max` are `#` and the rest `-`. Returns the screen (the result of the last `draw_text`). Pass floats for `current`/`max` if you want partial fill; integer division truncates. A `max` of 0 is a division-by-zero error.

### draw_score(screen, x, y, score)
Draws `Score: <score>` at (x, y). Returns the screen.

### draw_lives(screen, x, y, lives)
Draws one `<3 ` per life at (x, y). Returns the screen.

### draw_box(screen, x, y, w, h, style)
Draws a `w` by `h` box outline onto `screen` using a style dictionary with keys `tl`, `tr`, `bl`, `br`, `h`, `v` (use `BOX_SINGLE` or `BOX_DOUBLE`). Returns `naething`. Not the same as `draw_box` in [ascii_art](./ascii_art.md), which returns a list of strings.

### random_position(max_x, max_y)
Returns a random `{"x": ..., "y": ...}` with `0 <= x < max_x` and `0 <= y < max_y`. Builds the whole grid first, so it is slow for large areas.

### random_direction()
Returns one of `{"dx": 0, "dy": -1}` (up), `{"dx": 0, "dy": 1}` (down), `{"dx": -1, "dy": 0}` (left), `{"dx": 1, "dy": 0}` (right).

## Constants

- `BOX_SINGLE`: dictionary of `+`, `-`, `|` box characters.
- `BOX_DOUBLE`: dictionary of `#`, `=`, `#` box characters.
- Sprites (all `Sprite` objects): `SPRITE_PLAYER`, `SPRITE_ENEMY`, `SPRITE_HEART`, `SPRITE_STAR`, `SPRITE_TREE`, `SPRITE_HOUSE`, `SPRITE_GHOST`, `SPRITE_SKULL`, `SPRITE_HAGGIS`, `SPRITE_THISTLE`, `SPRITE_BAGPIPE`.

## Classes

### Screen
`Screen(width, height)` is a grid of single characters, initially filled with spaces. Fields: `width`, `height`, `buffer` (list of rows), `clear_char`. Drawing outside the grid is silently ignored.

- `clear()`: refill the grid with `clear_char`.
- `set_clear_char(ch)`: change the fill character (takes effect on the next `clear`).
- `set(x, y, ch)`: put `ch` at (x, y); out-of-range is ignored.
- `get(x, y)`: the character at (x, y), or `naething` if out of range.
- `draw_text(x, y, text)`: write `text` left to right from (x, y).
- `draw_line(x1, y1, x2, y2, ch)`: Bresenham line, both ends included.
- `draw_rect(x, y, w, h, ch)`: rectangle outline.
- `fill_rect(x, y, w, h, ch)`: filled rectangle.
- `draw_circle(cx, cy, r, ch)`: midpoint-circle outline.
- `draw_sprite(x, y, sprite)`: copy a `Sprite` in with its top-left at (x, y). Space characters in the sprite are transparent.
- `render()`: returns the whole grid as one string, rows joined by newlines.
- `display()`: prints `render()`.

### Sprite
`Sprite(art)`: `art` is either a string with `\n` line breaks or a list of line strings. Fields: `lines`, `width` (longest line), `height`.

- `get_lines()`, `get_width()`, `get_height()`: accessors.
- `flip_horizontal()`: returns a new `Sprite` with each line reversed (characters such as `/` are not swapped for `\`).
- `flip_vertical()`: returns a new `Sprite` with the lines in reverse order.

### Entity
`Entity(x, y)`: fields `x`, `y`, `vx`, `vy` (0), `sprite` (`naething`), `char` (`"@"`), `active` (`aye`), `tags` (empty list).

- `with_sprite(sprite)`, `with_char(ch)`, `with_velocity(vx, vy)`, `with_tag(tag)`: builder-style setters.
- `has_tag(tag)`: `aye` if the tag was added.
- `move(dx, dy)`: shift the position by an offset.
- `update()`: add the velocity to the position.
- `draw(screen)`: draws the sprite, or the single `char` if there is no sprite. Does nothing if `active` is `nae`.
- `collides_with(other)`: bounding-box overlap when both entities have sprites; otherwise `aye` only if both positions are identical. Note that if only one has a sprite, the point test is used.
- `distance_to(other)`: Manhattan distance `|dx| + |dy|`.

### GameState
`GameState()`: fields `score` (0), `level` (1), `lives` (3), `running` (`aye`), `paused` (`nae`), `data` (empty dictionary).

- `add_score(points)`, `next_level()`: bump the counters.
- `lose_life()`: one fewer life; `running` becomes `nae` at 0 lives or fewer.
- `game_over()`: set `running` to `nae`.
- `toggle_pause()`: flip `paused`.
- `set_data(key, value)`, `get_data(key)`: free-form storage; `get_data` returns `naething` for a missing key.

### Animation
`Animation(frames)`: `frames` is a list (usually of sprites). Fields: `frames`, `current_frame`, `frame_duration` (1), `frame_counter`, `looping` (`aye`).

- `with_duration(duration)`: number of `update()` calls each frame lasts.
- `no_loop()`: hold on the last frame instead of wrapping.
- `update()`: advance the frame counter and, when due, the frame.
- `get_current()`: the current frame. An empty frame list is an out-of-range index error.
- `reset()`: back to the first frame.

### Particle
`Particle(x, y, char)`: fields `x`, `y`, `vx`, `vy` (0), `char`, `life` (10), `alive`. `update()` moves it, uses up one life and sets `alive` to `nae` at 0.

### ParticleSystem
`ParticleSystem()`: field `particles`.

- `emit(x, y, count, chars)`: adds `count` particles at (x, y), each with a random character from the list `chars`, a fixed pseudo-random velocity pattern and a life of 5 to 9.
- `update()`: updates every particle and drops dead ones.
- `draw(screen)`: plots each particle on the screen.
- `clear()`: removes all particles.

## Example

```mdhavers
fetch "stdlib/game"

ken screen = make_screen(12, 5)
draw_box(screen, 0, 0, 12, 5, BOX_SINGLE)
screen.draw_text(2, 2, "Braw!")
ken hero = make_entity(8, 1).with_char("@")
hero.draw(screen)
ken state = make_game_state()
state.add_score(10)
screen.display()
blether f"Score: {state.score}"
```

## Compared with ascii_art

[ascii_art](./ascii_art.md) builds static pictures as lists of strings (boxes, banners, bars, patterns) that you print with `display_art`. `game` draws onto a mutable `Screen` grid you redraw every frame. Use `game` for anything interactive or animated and `ascii_art` for one-off text graphics. Do not fetch both in one program unless you accept that the later `fetch` replaces `BOX_SINGLE`, `BOX_DOUBLE` and `draw_box`.
