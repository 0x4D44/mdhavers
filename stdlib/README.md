# mdhavers Standard Library

**"A' yer tools in yin place!"**

The mdhavers standard library provides a comprehensive set of modules fer buildin' braw applications. Each module is written in pure mdhavers an' demonstrates the language's capabilities.

## Quick Start

Tae use a module, simply fetch it at the top o' yer file:

```braw
fetch "lib/colors"
fetch "lib/functional"
```

## Module Index

### Core Utilities

| Module | Description | Key Features |
|--------|-------------|--------------|
| **prelude** | Auto-loaded core functions | Basic list/dict operations, type conversion |
| **strings** | String manipulation | Advanced string functions |
| **maths** | Mathematical functions | Trigonometry, statistics, number theory |
| **collections** | Collection utilities | Advanced list/dict operations |

### Data Structures

| Module | Description | Key Features |
|--------|-------------|--------------|
| **structures** | Common data structures | Stack, Queue, Deque, Set, PriorityQueue, RingBuffer, LRUCache, Counter |

### Functional Programming

| Module | Description | Key Features |
|--------|-------------|--------------|
| **functional** | FP utilities | map, filter, reduce, compose, pipeline, memoize, Functor |

### Date & Time

| Module | Description | Key Features |
|--------|-------------|--------------|
| **dates** | Basic date operations | Date arithmetic, formatting |
| **datetime** | Full datetime support | DateTime class, Duration, Scottish day names, holidays |

### Text & Pattern Matching

| Module | Description | Key Features |
|--------|-------------|--------------|
| **patterns** | Design patterns | Builder, Factory, Observer, State patterns |
| **patterns_match** | Pattern matching | Glob matching, templates, tokenizer |

### Configuration & Logging

| Module | Description | Key Features |
|--------|-------------|--------------|
| **config** | Configuration handling | INI parsing, env vars, feature flags |
| **logging** | Structured logging | Log levels, Scottish mode (HAVERS, KEN, WATCH, FANKLE, DEID) |

### User Interface

| Module | Description | Key Features |
|--------|-------------|--------------|
| **menus** | Interactive menus | Menu, TextInput, NumberInput, ProgressBar, Form, Table |
| **colors** | ANSI terminal colors | Foreground, background, styles, gradients |

### File System

| Module | Description | Key Features |
|--------|-------------|--------------|
| **paths** | Path manipulation | basename, dirname, normalize, Path class, file type detection |

### Data Formats

| Module | Description | Key Features |
|--------|-------------|--------------|
| **json** | JSON handling | Parse, stringify, pretty print |

### Validation & Testing

| Module | Description | Key Features |
|--------|-------------|--------------|
| **validate** | Data validation | Email, phone, URL, credit card validators |
| **testing** | Unit testing | Test runner, assertions, benchmarks |
| **benchmark** | Performance testing | Timing, comparison, statistics |

### Async & Concurrency

| Module | Description | Key Features |
|--------|-------------|--------------|
| **promise** | Promise patterns | Promise, Result (Either), Option (Maybe), Lazy evaluation |
| **events** | Event system | EventEmitter, pub/sub |
| **statemachine** | State machines | StateMachine class, transitions |

### Security & Identity

| Module | Description | Key Features |
|--------|-------------|--------------|
| **crypto** | Cryptographic utilities | Hash functions, encoding, checksums |
| **uuid** | UUID generation | UUID v4, validation |

### Networking

| Module | Description | Key Features |
|--------|-------------|--------------|
| **http** | HTTP client utilities | URL parsing, Request/Response, Headers, Cookies, MockClient |

### Game Development

| Module | Description | Key Features |
|--------|-------------|--------------|
| **game** | ASCII game utilities | Screen buffer, Sprites, Entities, Animation, Particles, UI elements |

### Scottish Flavour

| Module | Description | Key Features |
|--------|-------------|--------------|
| **scots_lang** | Scottish language | Greetings, proverbs, weather words, name generator, text converter |
| **poetry** | Scottish poetry | Burns quotes, verse generation |

---

## Module Details

### prelude (Auto-loaded)

The prelude is automatically loaded an' provides essential functions:

```braw
# Type conversion
tae_string(42)       # "42"
tae_int("42")        # 42
tae_float("3.14")    # 3.14

# List operations
shove(list, item)    # Push tae list
yank(list)           # Pop fae list
heid(list)           # First element
tail(list)           # All but first
bum(list)            # Last element
scran(list, 0, 3)    # Slice

# String operations
wheesht(str)         # Trim whitespace
upper(str)           # Uppercase
lower(str)           # Lowercase
split(str, ",")      # Split string
join(list, ",")      # Join list

# Type checking
whit_kind(x)         # Get type name
```

### functional

Functional programming utilities:

```braw
fetch "lib/functional"

ken nums = [1, 2, 3, 4, 5]

# Core operations
ken doubled = map_list(nums, |x| x * 2)      # [2, 4, 6, 8, 10]
ken evens = filter_list(nums, is_even)        # [2, 4]
ken total = reduce(nums, |a, b| a + b, 0)     # 15

# Predicates
any(nums, is_even)   # aye
all(nums, is_positive)  # aye
find(nums, |x| x > 3)   # 4

# Composition
ken f = compose(double, increment)  # f(x) = double(increment(x))
ken g = pipeline([increment, double, square])
```

### structures

Common data structures:

```braw
fetch "lib/structures"

# Stack (LIFO)
ken stack = Stack()
stack.push(1).push(2).push(3)
stack.pop()   # 3
stack.peek()  # 2

# Queue (FIFO)
ken queue = Queue()
queue.enqueue("a").enqueue("b")
queue.dequeue()  # "a"

# Set
ken s = make_set([1, 2, 3, 4])
s.has(3)  # aye
s.union(other_set)
s.intersection(other_set)

# Counter
ken c = Counter(["a", "b", "a", "c", "a"])
c.get("a")  # 3
c.most_common(2)  # [{"key": "a", "count": 3}, ...]
```

### logging

Structured logging with Scottish flair:

```braw
fetch "lib/logging"

ken logger = Logger("myapp")
logger.scots_mode(aye)  # Use Scottish level names

logger.havers("Debug message")      # [HAVERS]
logger.ken_this("Info message")     # [KEN]
logger.watch_yersel("Warning")      # [WATCH]
logger.fankle("Error message")      # [FANKLE]
logger.deid("Fatal error")          # [DEID]

# Structured logging
ken struct_log = StructuredLogger("api")
struct_log.info("Request", {"path": "/users", "status": 200})
```

### colors

ANSI terminal colors:

```braw
fetch "lib/colors"

blether red("This is red!")
blether green("This is green!")
blether bold(blue("Bold blue!"))

# Background colors
blether bg_yellow(black("Warning!"))

# Styles
blether underline("Underlined text")
blether strike("Struck through")

# RGB colors (if terminal supports)
blether rgb(255, 128, 0, "Orange text!")
```

### scots_lang

Scottish language utilities:

```braw
fetch "lib/scots_lang"

# Random phrases
blether random_greeting()    # "Hullo there!" / "Whit like?" / ...
blether random_proverb()     # Scottish wisdom
blether random_farewell()    # "Haste ye back!" / ...

# Dictionary
whit_means("braw")      # "great, fine"
whit_means("haggis")    # "Scottish delicacy"

# Name generator
random_scots_name()         # "Angus MacDonald"
random_scots_name("female") # "Morag Campbell"
random_place_name()         # "Inverness" / "Aberdale" / ...

# Text converter
make_scots("I don't know")  # "I dinnae ken"
```

---

## Writing Yer Own Modules

Create a `.braw` file in the `stdlib/` or `examples/lib/` directory:

```braw
# mymodule.braw - Description o' yer module
# "A catchy Scottish phrase!"

# Yer code here...

kin MyClass {
    dae init() {
        masel.value = 0
    }

    dae do_something() {
        gie masel.value
    }
}

dae my_function(x) {
    gie x * 2
}

blether "Mymodule loaded! Ready tae go!"
```

Then use it:

```braw
fetch "lib/mymodule"  # or "mymodule" if in stdlib

ken obj = MyClass()
blether my_function(21)
```

---

## Scots Vocabulary Quick Reference

| English | mdhavers | Usage |
|---------|----------|-------|
| let/var | `ken` | `ken x = 42` |
| if | `gin` | `gin x > 0 { }` |
| else | `ither` | `} ither { }` |
| while | `whiles` | `whiles x > 0 { }` |
| for | `fer` | `fer x in list { }` |
| function | `dae` | `dae foo() { }` |
| return | `gie` | `gie value` |
| class | `kin` | `kin MyClass { }` |
| self/this | `masel` | `masel.x` |
| true | `aye` | `ken flag = aye` |
| false | `nae` | `ken flag = nae` |
| null | `naething` | `ken x = naething` |
| print | `blether` | `blether "Hello!"` |
| try | `hae_a_bash` | `hae_a_bash { }` |
| catch | `gin_it_gangs_wrang` | `gin_it_gangs_wrang e { }` |
| break | `brak` | `brak` |
| continue | `haud` | `haud` |
| import | `fetch` | `fetch "module"` |
| and | `an` | `x an y` |
| or | `or` | `x or y` |
| not | `nae` | `nae x` |

---

**Lang may yer lum reek!** 🏴󠁧󠁢󠁳󠁣󠁴󠁿
