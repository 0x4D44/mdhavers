# Keywords Reference

Complete reference of all mdhavers keywords with their Scots meanings and usage.

## Variable Declaration

### ken
**Meaning:** "To know" or "to understand"
**Usage:** Declare variables

```scots
ken name = "Hamish"
ken age = 30
ken is_scottish = aye
```

## Conditionals

### gin
**Meaning:** "If"
**Usage:** Start a conditional block

```scots
gin age >= 18 {
    blether "Ye're an adult!"
}
```

### ither
**Meaning:** "Other" or "else"
**Usage:** Alternative branch in conditionals

```scots
gin score >= 70 {
    blether "Ye passed!"
} ither {
    blether "Try again."
}
```

### than
**Meaning:** "Then"
**Usage:** Ternary expressions

```scots
ken result = gin x > 0 than "positive" ither "not positive"
```

## Loops

### whiles
**Meaning:** "While"
**Usage:** While loop

```scots
ken count = 0
whiles count < 5 {
    blether count
    count = count + 1
}
```

### fer
**Meaning:** "For"
**Usage:** For loop

```scots
fer i in 1..10 {
    blether i
}

fer item in my_list {
    blether item
}
```

### in
**Meaning:** "In"
**Usage:** Iteration in for loops

```scots
fer x in collection {
    # ...
}
```

### brak
**Meaning:** "Break"
**Usage:** Exit a loop early

```scots
fer i in 1..100 {
    gin i > 10 {
        brak
    }
}
```

### haud
**Meaning:** "Hold" (as in "hold on")
**Usage:** Continue to next iteration

```scots
fer i in 1..10 {
    gin i % 2 == 0 {
        haud  # Skip even numbers
    }
    blether i
}
```

## Functions

### dae
**Meaning:** "To do"
**Usage:** Define a function

```scots
dae greet(name) {
    blether f"Hullo, {name}!"
}
```

### gie
**Meaning:** "To give"
**Usage:** Return a value from a function

```scots
dae add(a, b) {
    gie a + b
}
```

## Boolean Values

### aye
**Meaning:** "Yes"
**Usage:** Boolean true

```scots
ken is_ready = aye
```

### nae
**Meaning:** "No" or "not"
**Usage:** Boolean false, or logical NOT

```scots
ken is_empty = nae
ken is_invalid = nae is_valid
```

### naething
**Meaning:** "Nothing"
**Usage:** Null value

```scots
ken result = naething
```

## Logical Operators

### an
**Meaning:** "And"
**Usage:** Logical AND

```scots
gin age >= 18 an has_license {
    blether "Can drive"
}
```

### or
**Meaning:** "Or"
**Usage:** Logical OR

```scots
gin is_admin or is_owner {
    blether "Has access"
}
```

## Input/Output

### blether
**Meaning:** "To chat" or "to talk"
**Usage:** Print output

```scots
blether "Hullo, World!"
blether 42
blether my_list
```

### speir
**Meaning:** "To ask"
**Usage:** Get user input

```scots
ken name = speir "Whit's yer name? "
```

## Classes

### kin
**Meaning:** "Family" or "type"
**Usage:** Define a class

```scots
kin Person {
    dae init(name) {
        masel.name = name
    }
}
```

### fae
**Meaning:** "From"
**Usage:** Class inheritance

```scots
kin Dog fae Animal {
    dae speak() {
        blether "Woof!"
    }
}
```

### masel
**Meaning:** "Myself"
**Usage:** Reference to current instance (like `this` or `self`)

```scots
kin Counter {
    dae init() {
        masel.count = 0
    }

    dae increment() {
        masel.count = masel.count + 1
    }
}
```

### thing
**Meaning:** "Thing"
**Usage:** Define a simple struct (record type)

```scots
thing Point {
    x,
    y
}
```

## Modules

### fetch
**Meaning:** "Fetch" (to get)
**Usage:** Import a module

```scots
fetch "lib/helpers"
```

### tae
**Meaning:** "To"
**Usage:** Import module with alias

```scots
fetch "lib/math" tae m
blether m["square"](5)
```

## Error Handling

### hae_a_bash
**Meaning:** "Have a bash" (try something)
**Usage:** Try block

```scots
hae_a_bash {
    ken result = risky_operation()
} gin_it_gangs_wrang err {
    blether f"Error: {err}"
}
```

### gin_it_gangs_wrang
**Meaning:** "If it goes wrong"
**Usage:** Catch block

```scots
hae_a_bash {
    # risky code
} gin_it_gangs_wrang error {
    # handle error
}
```

## Pattern Matching

### keek
**Meaning:** "Peek" or "look"
**Usage:** Match expression

```scots
keek day {
    whan 1 -> { blether "Monday" }
    whan 2 -> { blether "Tuesday" }
    whan _ -> { blether "Other day" }
}
```

### whan
**Meaning:** "When"
**Usage:** Match case

```scots
keek value {
    whan 0 -> { gie "zero" }
    whan x -> { gie f"got {x}" }
}
```

## Assertions

### mak_siccar
**Meaning:** "Make sure" (famously said by Robert the Bruce!)
**Usage:** Assert a condition

```scots
mak_siccar x > 0, "x must be positive"
mak_siccar len(list) > 0
```

## Type Checking

### is
**Meaning:** "Is"
**Usage:** Type checking (with is_a function)

```scots
gin is_a(x, "integer") {
    blether "It's a number!"
}
```

## Summary Table

| Keyword | Scots Meaning | English Equivalent |
|---------|--------------|-------------------|
| `ken` | to know | let/var |
| `gin` | if | if |
| `ither` | other/else | else |
| `than` | then | then (ternary) |
| `whiles` | while | while |
| `fer` | for | for |
| `in` | in | in |
| `brak` | break | break |
| `haud` | hold | continue |
| `dae` | to do | function |
| `gie` | to give | return |
| `aye` | yes | true |
| `nae` | no/not | false/not |
| `naething` | nothing | null |
| `an` | and | && |
| `or` | or | \|\| |
| `blether` | to chat | print |
| `speir` | to ask | input |
| `kin` | family/type | class |
| `fae` | from | extends |
| `masel` | myself | this/self |
| `thing` | thing | struct |
| `fetch` | fetch | import |
| `tae` | to | as |
| `hae_a_bash` | have a bash | try |
| `gin_it_gangs_wrang` | if it goes wrong | catch |
| `keek` | peek/look | match |
| `whan` | when | case |
| `mak_siccar` | make sure | assert |
| `is` | is | instanceof |
