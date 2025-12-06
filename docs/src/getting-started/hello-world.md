# Hello World

Let's write your first mdhavers program!

## Your First Program

Create a new file called `hello.braw` (mdhavers files use the `.braw` extension - it's Scots for "good" or "fine"):

```scots
# My first mdhavers program
blether "Hullo, World!"
```

Run it:
```bash
mdhavers hello.braw
```

Output:
```
Hullo, World!
```

Congratulations - you've written your first Scots program!

## What Just Happened?

Let's break down the code:

- `#` starts a comment (just like Python)
- `blether` is the print command - it means "to chat" or "to talk" in Scots
- Text in quotes is a string

## A Wee Bit More

Let's make it more interesting:

```scots
# Ask for a name and greet them
ken name = speir "Whit's yer name? "
blether f"Hullo, {name}! Hoo's it gaun?"
```

Run it:
```bash
mdhavers hello.braw
```

```
Whit's yer name? Hamish
Hullo, Hamish! Hoo's it gaun?
```

Here we used:
- `ken` - to declare a variable ("ken" means "to know" in Scots)
- `speir` - to ask for input ("speir" means "to ask")
- `f"..."` - an f-string for interpolating the variable

## Try the REPL

mdhavers has an interactive mode called the REPL (Read-Eval-Print Loop). Start it:

```bash
mdhavers repl
```

Or simply:
```bash
mdhavers
```

You'll see:
```
🏴󠁧󠁢󠁳󠁣󠁴󠁿 mdhavers REPL - Pure Braw!
Type 'help' fer commands, 'quit' tae leave

mdhavers>
```

Try some expressions:
```
mdhavers> 2 + 2
4
mdhavers> ken x = 42
mdhavers> x * 2
84
mdhavers> blether "Testing!"
Testing!
mdhavers> quit
```

The REPL is perfect for experimenting and learning!

## Error Messages in Scots

What happens when something goes wrong? mdhavers gives you friendly Scottish error messages:

```scots
blether undefined_variable
```

```
Och! Ah dinnae ken whit 'undefined_variable' is at line 1, column 9
```

Try dividing by zero:
```scots
ken x = 10 / 0
```

```
Ye numpty! Tryin' tae divide by zero at line 1
```

These friendly messages make debugging a bit more enjoyable!

## Common Scots Keywords

Here's a quick reference of the keywords you'll use most often:

| Scots | English | Usage |
|-------|---------|-------|
| `ken` | know | Declare variables |
| `blether` | talk/chat | Print output |
| `speir` | ask | Get input |
| `gin` | if | Conditions |
| `ither` | else | Alternative branch |
| `fer` | for | Loops |
| `dae` | do | Define functions |
| `gie` | give | Return values |
| `aye` | yes | True |
| `nae` | no | False |

## What's Next?

Now that you've got the basics, you can:

1. Set up your [editor](./editor-setup.md) with syntax highlighting
2. Learn about [running programs](./running-programs.md) from the command line
3. Dive into the [language basics](../learn/01-basics.md)

Welcome to mdhavers - have fun learning to code in Scots!
