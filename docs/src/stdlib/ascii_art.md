# ascii_art

Text graphics. Most functions return "art": a list of strings, one per line, which you print with `display_art`. Included are Scottish symbols, boxes and banners, progress bars, spinner frames, lines, block-letter text, speech bubbles, patterns, and helpers to centre, align and combine art.

Import: `fetch "stdlib/ascii_art"`

Pure mdhavers; no native built-ins. Output uses Unicode box and block characters, so the terminal needs UTF-8. Loading the module prints `ASCII Art module loaded! Drawin' pictures wi' letters!`. No function hurls its own errors; the one runtime failure worth knowing is listed under `vertical_gradient`. It defines top-level `BOX_SINGLE`, `BOX_DOUBLE` and `draw_box`, which clash with [game](./game.md) if both are fetched.

## Constants

- `BOX_SINGLE`, `BOX_DOUBLE`, `BOX_ROUNDED`: dictionaries of box characters with keys `tl tr bl br h v lt rt tt bt cross`.
- `SHADES`: `["░", "▒", "▓", "█"]`.
- `BLOCKS`: `full`, `upper`, `lower`, `left`, `right` block characters.
- `SPINNER_FRAMES`, `SPINNER_DOTS`, `SPINNER_SCOTS`: frame lists for `spinner_frame`.
- `BLOCK_LETTERS`: five-line glyphs for `A` to `Z` and the space, used by `big_text`.

## Functions

### thistle()
Art (list of 8 lines) of a thistle.

### saltire()
Art of the Saltire.

### highland_coo()
Art of a Highland cow (6 lines).

### bagpipes()
Art of bagpipes.

### castle()
Art of a castle.

### haggis_creature()
Art of a wild haggis.

### whisky_glass()
Art of a glass of whisky.

### scottish_flag()
Art of a boxed saltire flag.

### tartan_pattern(width = 20, height = 10)
Art of a tartan-style shaded pattern of the given size.

### display_art(art)
Prints each line of the art. Returns `naething`.

### display_art_centered(art, width = 60)
Prints each line indented to be centred in `width` columns (never negative indent).

### draw_box(width, height, style = "single")
Art of an empty box of the given outer size. `style` is `"single"`, `"double"` or `"rounded"`; any other value gives the single style.

### draw_box_with_title(title, width = 40, style = "double")
Art of a three-line box with `title` centred inside. The title is not truncated, so make `width` at least `len(title) + 2`.

### progress_bar(value, max_val = 100, width = 30, filled = "█", empty = "░")
Returns a string like `[█████░░░░░]`. The ratio is clamped to 0..1. Pass floats: with two integers the division truncates, so `progress_bar(50, 100, 10)` shows an empty bar and only a full value fills it.

### scots_progress_bar(value, max_val = 100, width = 30)
A `progress_bar` followed by the percentage and a Scots message, for example `[█████░░░░░] 50% - Mair than hauf done!`. The same integer-division caveat applies to the bar (the percentage is correct).

### spinner_frame(frame_num, spinner_type = "default")
The frame for `frame_num` (wraps around). `spinner_type` is `"default"` (`| / - \`), `"dots"` or `"scots"`.

### horizontal_line(width = 40, char = "─")
A string of `char` repeated `width` times.

### double_line(width = 40)
A line of `═`.

### dotted_line(width = 40)
Alternating `·` and spaces.

### wavy_line(width = 40)
Alternating `~` and `˜`.

### big_text(text)
Art (5 lines) of `text` in block letters. The text is upper-cased; only `A` to `Z` and space have glyphs, and anything else (digits, punctuation) renders as a blank.

### has_key(dict, key)
`aye` if `key` is one of the dictionary's keys.

### banner(text, style = "double")
Art (3 lines) of `text` in a box, with one space of padding each side.

### scots_banner(text)
Art (3 lines) of `text` in a double box decorated with emoji. Needs a terminal that renders emoji.

### vertical_gradient(width, height)
Art of four bands from light to dark shading. `height` must be 4 or more: smaller values divide by zero (`Tryin' tae divide by zero`).

### checkerboard(width, height, char1 = "█", char2 = "░")
Art of alternating characters.

### diagonal_pattern(width, height)
Art of alternating `/` and `\`.

### frame_art(inner_art, style = "double")
Wraps existing art in a box sized to its longest line. Style as for `draw_box`.

### shadow_box(width, height)
A single-style box with a `░` drop shadow right and below.

### speech_bubble(text)
Art (5 lines) of `< text >` with a tail.

### thought_bubble(text)
Art (6 lines) of `( text )` with trailing circles.

### repeat_char(char, count)
A string of `char` repeated `count` times.

### center_text(text, width)
`text` indented with spaces so it is centred in `width`. Returns one string, with no right padding.

### right_align(text, width)
`text` indented so it ends at column `width`.

### combine_art_horizontal(art1, art2, spacing = 2)
Places two pieces of art side by side; the shorter is padded with blank lines, and the left art is padded to its widest line before `spacing` spaces.

### combine_art_vertical(art1, art2, spacing = 1)
Stacks two pieces of art with `spacing` blank lines between.

## Example

```mdhavers
fetch "stdlib/ascii_art"

display_art(banner("Braw", "single"))
display_art(draw_box_with_title("Hi", 10))
blether progress_bar(50.0, 100.0, 10)
display_art(speech_bubble("Hullo"))
display_art(combine_art_horizontal(["a", "b"], ["X"]))
```

## Compared with game

[game](./game.md) draws on a mutable `Screen` grid that you redraw each frame and is meant for interactive programs. `ascii_art` produces finished, static lists of lines. Use `ascii_art` for banners, menus and one-off pictures and `game` for anything that moves. Both define `BOX_SINGLE`, `BOX_DOUBLE` and `draw_box` with different shapes (`ascii_art` returns lines, `game` takes a screen and draws on it), so do not fetch both into one program.
