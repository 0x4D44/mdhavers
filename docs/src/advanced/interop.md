# JavaScript Compilation

Compile mdhavers code to JavaScript for browser and Node.js environments.

## Basic Compilation

Compile a `.braw` file to JavaScript:

```bash
# Print to stdout
mdhavers compile program.braw

# Write to file
mdhavers compile program.braw -o program.js
```

## Example

Given this mdhavers code:

```scots
# greeter.braw
dae greet(name) {
    gie f"Hullo, {name}!"
}

ken message = greet("World")
blether message
```

The compiler generates JavaScript:

```javascript
// Generated JavaScript
function greet(name) {
    return `Hullo, ${name}!`;
}

let message = greet("World");
console.log(message);
```

## Running Compiled Code

### In Node.js

```bash
mdhavers compile program.braw -o program.js
node program.js
```

### In Browser

```html
<!DOCTYPE html>
<html>
<head>
    <title>mdhavers App</title>
</head>
<body>
    <script src="program.js"></script>
</body>
</html>
```

## Language Feature Mapping

### Variables and Types

| mdhavers | JavaScript |
|----------|------------|
| `ken x = 5` | `let x = 5;` |
| `aye` | `true` |
| `nae` | `false` |
| `naething` | `null` |
| `[1, 2, 3]` | `[1, 2, 3]` |
| `{"a": 1}` | `{a: 1}` |

### Control Flow

| mdhavers | JavaScript |
|----------|------------|
| `gin x > 0 { ... }` | `if (x > 0) { ... }` |
| `ither { ... }` | `else { ... }` |
| `whiles x < 10 { ... }` | `while (x < 10) { ... }` |
| `fer i in 0..10 { ... }` | `for (let i = 0; i < 10; i++) { ... }` |
| `brak` | `break;` |
| `haud` | `continue;` |

### Functions

| mdhavers | JavaScript |
|----------|------------|
| `dae foo(x) { gie x * 2 }` | `function foo(x) { return x * 2; }` |
| `\|x\| x * 2` | `(x) => x * 2` |
| `blether x` | `console.log(x);` |

### Classes

```scots
# mdhavers
kin Point {
    dae init(x, y) {
        masel.x = x
        masel.y = y
    }

    dae distance() {
        gie sqrt(masel.x * masel.x + masel.y * masel.y)
    }
}
```

```javascript
// JavaScript
class Point {
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }

    distance() {
        return Math.sqrt(this.x * this.x + this.y * this.y);
    }
}
```

## Built-in Function Mapping

Many mdhavers built-ins map to JavaScript equivalents:

| mdhavers | JavaScript |
|----------|------------|
| `len(x)` | `x.length` |
| `blether(x)` | `console.log(x)` |
| `upper(s)` | `s.toUpperCase()` |
| `lower(s)` | `s.toLowerCase()` |
| `split(s, d)` | `s.split(d)` |
| `join(a, d)` | `a.join(d)` |
| `abs(n)` | `Math.abs(n)` |
| `sqrt(n)` | `Math.sqrt(n)` |
| `floor(n)` | `Math.floor(n)` |
| `ceil(n)` | `Math.ceil(n)` |

## Web Integration

### DOM Manipulation

For browser use, you can interface with JavaScript APIs:

```scots
# This requires JavaScript glue code
dae update_page(content) {
    # Will be compiled to JavaScript that can access DOM
    blether content
}
```

### Event Handling

Create a JavaScript wrapper to connect mdhavers functions to events:

```javascript
// glue.js
import { handleClick } from './compiled.js';

document.getElementById('btn').addEventListener('click', () => {
    handleClick();
});
```

## Limitations

The JavaScript compiler has some limitations compared to the interpreter:

1. **File I/O**: Browser JavaScript can't directly access the file system
2. **User Input**: `speir` doesn't work in browsers (use DOM input instead)
3. **Some Built-ins**: Not all Scots-specific built-ins may be available

## Build Workflow

For production use:

```bash
# 1. Compile mdhavers to JavaScript
mdhavers compile src/main.braw -o dist/app.js

# 2. Bundle with your build tool (optional)
# Using esbuild, webpack, rollup, etc.

# 3. Minify for production (optional)
# Using terser, uglify, etc.
```

## Example: Simple Web App

**counter.braw:**
```scots
ken count = 0

dae increment() {
    count = count + 1
    gie count
}

dae decrement() {
    count = count - 1
    gie count
}

dae get_count() {
    gie count
}
```

**index.html:**
```html
<!DOCTYPE html>
<html>
<head>
    <title>Counter</title>
</head>
<body>
    <h1>Count: <span id="count">0</span></h1>
    <button onclick="inc()">+</button>
    <button onclick="dec()">-</button>

    <script src="counter.js"></script>
    <script>
        function inc() {
            document.getElementById('count').textContent = increment();
        }
        function dec() {
            document.getElementById('count').textContent = decrement();
        }
    </script>
</body>
</html>
```

Build and serve:
```bash
mdhavers compile counter.braw -o counter.js
python -m http.server 8000
# Open http://localhost:8000
```

## Debugging Compiled Code

The generated JavaScript maintains readable structure:

1. **Source maps**: Not currently supported, but code is readable
2. **Variable names**: Preserved from mdhavers
3. **Function names**: Preserved from mdhavers
4. **Comments**: Can be preserved with `--comments` flag

## Future: WASM Compilation

A WebAssembly backend is under development for:
- Better performance
- Smaller bundle sizes
- Cross-platform compatibility

See `src/wasm_compiler.rs` for the experimental WASM compiler.
