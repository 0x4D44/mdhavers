// mdhavers runtime - pure havers, but working havers!
const __havers = {
  len: (x) => {
    if (typeof x === 'string' || Array.isArray(x)) return x.length;
    if (x && typeof x === 'object') return Object.keys(x).length;
    throw new Error('Och! Cannae get length o\' that!');
  },
  whit_kind: (x) => {
    if (x === null || x === undefined) return 'naething';
    if (Array.isArray(x)) return 'list';
    if (typeof x === 'object') return 'dict';
    return typeof x;
  },
  tae_string: (x) => String(x),
  tae_int: (x) => {
    const n = parseInt(x, 10);
    if (isNaN(n)) throw new Error(`Cannae turn '${x}' intae an integer`);
    return n;
  },
  tae_float: (x) => {
    const n = parseFloat(x);
    if (isNaN(n)) throw new Error(`Cannae turn '${x}' intae a float`);
    return n;
  },
  shove: (arr, val) => { arr.push(val); },
  yank: (arr) => {
    if (arr.length === 0) throw new Error('Cannae yank fae an empty list!');
    return arr.pop();
  },
  keys: (obj) => Object.keys(obj),
  values: (obj) => Object.values(obj),
  range: (start, end) => {
    const result = [];
    for (let i = start; i < end; i++) result.push(i);
    return result;
  },
  abs: Math.abs,
  min: Math.min,
  max: Math.max,
  floor: Math.floor,
  ceil: Math.ceil,
  round: Math.round,
  sqrt: Math.sqrt,
  split: (str, delim) => str.split(delim),
  join: (arr, delim) => arr.join(delim),
  contains: (container, item) => {
    if (typeof container === 'string') return container.includes(item);
    if (Array.isArray(container)) return container.includes(item);
    if (typeof container === 'object') return item in container;
    return false;
  },
  reverse: (x) => {
    if (typeof x === 'string') return x.split('').reverse().join('');
    if (Array.isArray(x)) return [...x].reverse();
    throw new Error('reverse() expects a list or string');
  },
  sort: (arr) => [...arr].sort((a, b) => {
    if (typeof a === 'number' && typeof b === 'number') return a - b;
    return String(a).localeCompare(String(b));
  }),
  blether: console.log,
  speir: (prompt) => {
    if (typeof window !== 'undefined' && typeof window.prompt === 'function') {
      return window.prompt(String(prompt)) || "";
    }
    return "";
  },
  heid: (x) => {
    if (typeof x === 'string' || Array.isArray(x)) {
      if (x.length === 0) throw new Error('Cannae get heid o\' an empty list!');
      return x[0];
    }
    throw new Error('heid() expects a list or string');
  },
  tail: (x) => {
    if (typeof x === 'string') return x.slice(1);
    if (Array.isArray(x)) return x.slice(1);
    throw new Error('tail() expects a list or string');
  },
  bum: (x) => {
    if (typeof x === 'string' || Array.isArray(x)) {
      if (x.length === 0) throw new Error('Cannae get bum o\' an empty list!');
      return x[x.length - 1];
    }
    throw new Error('bum() expects a list or string');
  },
  scran: (x, start, end) => {
    if (typeof x === 'string' || Array.isArray(x)) return x.slice(start, end);
    throw new Error('scran() expects a list or string');
  },
  slap: (a, b) => {
    if (typeof a === 'string' && typeof b === 'string') return a + b;
    if (Array.isArray(a) && Array.isArray(b)) return [...a, ...b];
    throw new Error('slap() expects two lists or two strings');
  },
  sumaw: (arr) => {
    if (!Array.isArray(arr)) throw new Error('sumaw() expects a list');
    return arr.reduce((a, b) => a + b, 0);
  },
  coont: (x, item) => {
    if (typeof x === 'string') return x.split(item).length - 1;
    if (Array.isArray(x)) return x.filter(e => e === item).length;
    throw new Error('coont() expects a list or string');
  },
  wheesht: (str) => String(str).trim(),
  upper: (str) => String(str).toUpperCase(),
  lower: (str) => String(str).toLowerCase(),
  shuffle: (arr) => {
    if (!Array.isArray(arr)) throw new Error('shuffle() expects a list');
    const result = [...arr];
    for (let i = result.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [result[i], result[j]] = [result[j], result[i]];
    }
    return result;
  },
  slice: (x, start, end, step) => {
    const len = x.length;
    const isStr = typeof x === 'string';
    const arr = isStr ? x.split('') : x;
    if (step === 0) throw new Error('Slice step cannae be zero, ya dafty!');
    // Handle defaults based on step direction
    const s = start !== null ? (start < 0 ? Math.max(len + start, 0) : Math.min(start, len)) : (step > 0 ? 0 : len - 1);
    const e = end !== null ? (end < 0 ? Math.max(len + end, step > 0 ? 0 : -1) : Math.min(end, len)) : (step > 0 ? len : -len - 1);
    const result = [];
    if (step > 0) {
      for (let i = s; i < e; i += step) result.push(arr[i]);
    } else {
      for (let i = s; i > e; i += step) result.push(arr[i]);
    }
    return isStr ? result.join('') : result;
  },
  // Timing functions
  noo: () => Date.now(),
  tick: () => {
    if (typeof process !== 'undefined' && process.hrtime) {
      const [s, ns] = process.hrtime();
      return s * 1e9 + ns;
    }
    return Date.now() * 1e6; // Fallback for browser
  },
  bide: (ms) => {
    const end = Date.now() + ms;
    while (Date.now() < end) {} // Busy wait (sync)
  },
  // Higher-order functions
  gaun: (arr, fn) => arr.map(fn),
  sieve: (arr, fn) => arr.filter(fn),
  tumble: (arr, init, fn) => arr.reduce(fn, init),
  aw: (arr, fn) => arr.every(fn),
  ony: (arr, fn) => arr.some(fn),
  hunt: (arr, fn) => arr.find(fn),
};

const { len, whit_kind, tae_string, tae_int, tae_float, shove, yank, keys, values, range, abs, min, max, floor, ceil, round, sqrt, split, join, contains, reverse, sort, blether, speir, heid, tail, bum, scran, slap, sumaw, coont, wheesht, upper, lower, shuffle, noo, tick, bide, gaun, sieve, tumble, aw, ony, hunt } = __havers;

let BOARD_WIDTH = 10;
let BOARD_HEIGHT = 20;
let CELL_SIZE = 30;
let SHAPES = {"I": [[0, 0], [0, 1], [0, 2], [0, 3]], "O": [[0, 0], [0, 1], [1, 0], [1, 1]], "T": [[0, 0], [0, 1], [0, 2], [1, 1]], "S": [[0, 1], [0, 2], [1, 0], [1, 1]], "Z": [[0, 0], [0, 1], [1, 1], [1, 2]], "J": [[0, 0], [1, 0], [1, 1], [1, 2]], "L": [[0, 2], [1, 0], [1, 1], [1, 2]]};
let COLOURS = {"I": "#00CED1", "O": "#FFD700", "T": "#9370DB", "S": "#32CD32", "Z": "#FF4500", "J": "#4169E1", "L": "#FF8C00"};
let SHAPE_NAMES = ["I", "O", "T", "S", "Z", "J", "L"];
class TetrisGame {
  constructor() {
    (this.board = this.create_empty_board());
    (this.current_piece = null);
    (this.current_shape = "");
    (this.current_row = 0);
    (this.current_col = 0);
    (this.current_rotation = 0);
    (this.score = 0);
    (this.level = 1);
    (this.lines_cleared = 0);
    (this.game_over = false);
    (this.paused = false);
    this.spawn_piece();
  }
  create_empty_board() {
    let board = [];
    for (const row of __havers.range(0, BOARD_HEIGHT)) {
      let line = [];
      for (const col of __havers.range(0, BOARD_WIDTH)) {
        shove(line, 0);
      }
      shove(board, line);
    }
    return board;
  }
  spawn_piece() {
    let idx = jammy(0, (len(SHAPE_NAMES) - 1));
    (this.current_shape = SHAPE_NAMES[idx]);
    (this.current_piece = SHAPES[this.current_shape]);
    (this.current_rotation = 0);
    (this.current_row = 0);
    (this.current_col = (((BOARD_WIDTH - 4)) / 2));
    if (this.check_collision(0, 0)) {
      (this.game_over = true);
    }
  }
  get_rotated_piece() {
    let piece = this.current_piece;
    let rotated = [];
    for (const cell of piece) {
      let r = cell[0];
      let c = cell[1];
      if ((this.current_rotation === 1)) {
        shove(rotated, [c, (3 - r)]);
      } else       if ((this.current_rotation === 2)) {
        shove(rotated, [(3 - r), (3 - c)]);
      } else       if ((this.current_rotation === 3)) {
        shove(rotated, [(3 - c), r]);
      } else {
        shove(rotated, [r, c]);
      }


    }
    return rotated;
  }
  check_collision(row_offset, col_offset) {
    let piece = this.get_rotated_piece();
    for (const cell of piece) {
      let new_row = ((this.current_row + cell[0]) + row_offset);
      let new_col = ((this.current_col + cell[1]) + col_offset);
      if (((new_row < 0) || (new_row >= BOARD_HEIGHT))) {
        return true;
      }
      if (((new_col < 0) || (new_col >= BOARD_WIDTH))) {
        return true;
      }
      if ((this.board[new_row][new_col] !== 0)) {
        return true;
      }
    }
    return false;
  }
  move_left() {
    if (((!this.game_over) && (!this.paused))) {
      if ((!this.check_collision(0, (-1)))) {
        (this.current_col = (this.current_col - 1));
      }
    }
  }
  move_right() {
    if (((!this.game_over) && (!this.paused))) {
      if ((!this.check_collision(0, 1))) {
        (this.current_col = (this.current_col + 1));
      }
    }
  }
  move_down() {
    if (((!this.game_over) && (!this.paused))) {
      if ((!this.check_collision(1, 0))) {
        (this.current_row = (this.current_row + 1));
        return true;
      } else {
        this.lock_piece();
        this.clear_lines();
        this.spawn_piece();
        return false;
      }
    }
    return false;
  }
  hard_drop() {
    if (((!this.game_over) && (!this.paused))) {
      let dropped = 0;
      while ((!this.check_collision(1, 0))) {
        (this.current_row = (this.current_row + 1));
        let dropped = (dropped + 1);
      }
      (this.score = (this.score + ((dropped * 2))));
      this.lock_piece();
      this.clear_lines();
      this.spawn_piece();
    }
  }
  rotate() {
    if (((!this.game_over) && (!this.paused))) {
      let old_rotation = this.current_rotation;
      (this.current_rotation = (((this.current_rotation + 1)) % 4));
      if (this.check_collision(0, 0)) {
        let kicks = [0, 1, (-1), 2, (-2)];
        let kicked = false;
        for (const kick of kicks) {
          if ((!this.check_collision(0, kick))) {
            (this.current_col = (this.current_col + kick));
            let kicked = true;
            break;
          }
        }
        if ((!kicked)) {
          (this.current_rotation = old_rotation);
        }
      }
    }
  }
  lock_piece() {
    let piece = this.get_rotated_piece();
    let colour = COLOURS[this.current_shape];
    for (const cell of piece) {
      let row = (this.current_row + cell[0]);
      let col = (this.current_col + cell[1]);
      if (((((row >= 0) && (row < BOARD_HEIGHT)) && (col >= 0)) && (col < BOARD_WIDTH))) {
        (this.board[row][col] = colour);
      }
    }
  }
  clear_lines() {
    let lines_to_clear = [];
    for (const row of __havers.range(0, BOARD_HEIGHT)) {
      let complete = true;
      for (const col of __havers.range(0, BOARD_WIDTH)) {
        if ((this.board[row][col] === 0)) {
          let complete = false;
          break;
        }
      }
      if (complete) {
        shove(lines_to_clear, row);
      }
    }
    for (const line of lines_to_clear) {
      for (const row of __havers.range(line, 0)) {
        if ((row > 0)) {
          for (const col of __havers.range(0, BOARD_WIDTH)) {
            (this.board[row][col] = this.board[(row - 1)][col]);
          }
        } else {
          for (const col of __havers.range(0, BOARD_WIDTH)) {
            (this.board[0][col] = 0);
          }
        }
      }
    }
    let num_lines = len(lines_to_clear);
    if ((num_lines > 0)) {
      (this.lines_cleared = (this.lines_cleared + num_lines));
      let points = {1: 100, 2: 300, 3: 500, 4: 800};
      (this.score = (this.score + ((points[num_lines] * this.level))));
      (this.level = (((this.lines_cleared / 10)) + 1));
    }
  }
  toggle_pause() {
    if ((!this.game_over)) {
      (this.paused = (!this.paused));
    }
  }
  reset() {
    (this.board = this.create_empty_board());
    (this.score = 0);
    (this.level = 1);
    (this.lines_cleared = 0);
    (this.game_over = false);
    (this.paused = false);
    this.spawn_piece();
  }
  get_ghost_row() {
    let ghost_row = this.current_row;
    while ((!this.check_collision_at((ghost_row + 1), this.current_col))) {
      let ghost_row = (ghost_row + 1);
    }
    return ghost_row;
  }
  check_collision_at(row, col) {
    let piece = this.get_rotated_piece();
    let old_row = this.current_row;
    let old_col = this.current_col;
    (this.current_row = row);
    (this.current_col = col);
    let result = this.check_collision(0, 0);
    (this.current_row = old_row);
    (this.current_col = old_col);
    return result;
  }
  get_drop_speed() {
    let base_speed = 1000;
    let speed = (base_speed - ((((this.level - 1)) * 100)));
    if ((speed < 100)) {
      return 100;
    }
    return speed;
  }
}
let game = new TetrisGame();
function new_game() {
  game.reset();
}
function tick() {
  game.move_down();
}
function move_left() {
  game.move_left();
}
function move_right() {
  game.move_right();
}
function move_down() {
  game.move_down();
}
function hard_drop() {
  game.hard_drop();
}
function rotate() {
  game.rotate();
}
function toggle_pause() {
  game.toggle_pause();
}
function get_state() {
  let state = {"board": game.board, "current_piece": game.get_rotated_piece(), "current_shape": game.current_shape, "current_row": game.current_row, "current_col": game.current_col, "ghost_row": game.get_ghost_row(), "score": game.score, "level": game.level, "lines": game.lines_cleared, "game_over": game.game_over, "paused": game.paused, "colour": COLOURS[game.current_shape]};
  return state;
}
function get_colours() {
  return COLOURS;
}
function get_dimensions() {
  return {"width": BOARD_WIDTH, "height": BOARD_HEIGHT, "cell_size": CELL_SIZE};
}

export { game, new_game, tick, move_left, move_right, move_down, hard_drop, rotate, toggle_pause, get_state, get_colours, get_dimensions };
