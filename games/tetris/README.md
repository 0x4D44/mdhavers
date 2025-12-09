# mdhavers Tetris

A Tetris implementation written in mdhavers, the Scots programming language.

## Playing the Game

### Browser Version
Open `index.html` in your browser to play the game directly.

### Controls
- **Arrow Left/Right**: Move piece
- **Arrow Down**: Soft drop
- **Arrow Up**: Rotate
- **Space**: Hard drop
- **P**: Pause/Resume

### Mobile
On mobile devices, use the on-screen buttons.

## Files

- `tetris.braw` - The game logic written in mdhavers
- `index.html` - Browser-based game interface running the compiled code

## Game Features

- Classic Tetris gameplay
- 7 tetromino pieces with Scottish-themed colours:
  - **I** (Turquoise) - Like the Scottish sea
  - **O** (Gold) - Like whisky
  - **T** (Purple) - Like heather
  - **S** (Green) - Like the Highlands
  - **Z** (Orange-red) - Like a sunset
  - **J** (Royal blue) - Like the Saltire
  - **L** (Orange) - Like Irn-Bru
- Ghost piece showing where piece will land
- Hard drop for quick placement
- Wall kicks for rotation
- Level progression (speeds up every 10 lines)
- Scoring:
  - 1 line: 100 points
  - 2 lines: 300 points
  - 3 lines: 500 points
  - 4 lines (Tetris!): 800 points
  - Hard drop bonus: 2 points per cell dropped

## About mdhavers

mdhavers is a Scots programming language where:
- `ken` = let/var (I know)
- `gin` = if (if/when)
- `ither` = else
- `dae` = function (do)
- `gie` = return (give)
- `kin` = class (family)
- `masel` = self (myself)
- `aye` = true
- `nae` = false/not

See the [mdhavers documentation](../../docs/book/) for more.

## Embedding in Substack

To embed this Tetris game in a Substack article:

1. Host the files on a web server (GitHub Pages, Netlify, etc.)
2. Use Substack's embed feature with the hosted URL
3. Readers can play directly in the article!
