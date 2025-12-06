# mdhavers

**A Scots Programming Language - Pure havers, but working havers!**

mdhavers is a dynamically-typed programming language that uses Scots vocabulary for its keywords and produces error messages in Scots dialect. It's a fully-featured language with an interpreter, a JavaScript compiler, and a friendly REPL.

## What Does "Havers" Mean?

In Scots, "havers" means nonsense or foolish talk. So "mdhavers" is a playful acknowledgment that a programming language with Scots keywords might seem a bit daft - but it actually works!

## A Quick Taste

```scots
# Hello World in mdhavers
blether "Hullo, World!"

# Variables use 'ken' (to know)
ken name = "Hamish"
ken age = 42

# Functions use 'dae' (to do) and 'gie' (to give/return)
dae greet(person) {
    blether f"Hullo, {person}! Hoo's it gaun?"
}

# Conditions use 'gin' (if) and 'ither' (else)
gin age >= 18 {
    blether "Ye're auld enough fer a dram!"
} ither {
    blether "Nae whisky fer ye yet!"
}

# Loops use 'fer' (for) and 'whiles' (while)
fer i in 1..5 {
    blether f"Number {i}"
}

# Higher-order functions with Scots names
ken numbers = [1, 2, 3, 4, 5]
ken doubled = gaun(numbers, |x| x * 2)  # gaun = map
ken evens = sieve(numbers, |x| x % 2 == 0)  # sieve = filter
```

## Why mdhavers?

1. **Learn programming with a smile** - The Scots vocabulary makes coding memorable and fun
2. **Real programming concepts** - Despite the playful keywords, mdhavers teaches proper programming fundamentals
3. **Scots heritage** - Celebrate Scotland's rich linguistic tradition while coding
4. **Friendly error messages** - When things go wrong, you'll get errors like "Och! Ah dinnae ken whit 'xyz' is" instead of cryptic messages

## Features

- **Dynamic typing** with integers, floats, strings, booleans, lists, and dictionaries
- **First-class functions** including lambdas and closures
- **Object-oriented programming** with classes and inheritance
- **Functional programming** with map, filter, reduce (gaun, sieve, tumble)
- **Pattern matching** with the `keek`/`whan` syntax
- **Error handling** with `hae_a_bash`/`gin_it_gangs_wrang` (try/catch)
- **Module system** for organizing larger programs
- **JavaScript compilation** for running in browsers
- **Interactive REPL** for experimenting

## Getting Started

Ready to start coding in Scots? Head to the [Installation](./getting-started/installation.md) guide to set up mdhavers on your system, then write your [first program](./getting-started/hello-world.md)!

## Quick Links

- [Installation Guide](./getting-started/installation.md)
- [Language Basics](./learn/01-basics.md)
- [Keyword Reference](./reference/keywords.md)
- [Scots Glossary](./scots-glossary.md)

---

*"This is havers, but it's working havers!"*
