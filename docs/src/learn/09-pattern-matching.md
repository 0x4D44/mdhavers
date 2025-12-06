# Pattern Matching

Match values elegantly with `keek` (peek/look) and `whan` (when).

## Basic Pattern Matching

Use `keek` to match a value against patterns:

```scots
ken day = 3

keek day {
    whan 1 -> { blether "Monday" }
    whan 2 -> { blether "Tuesday" }
    whan 3 -> { blether "Wednesday" }
    whan 4 -> { blether "Thursday" }
    whan 5 -> { blether "Friday" }
    whan 6 -> { blether "Saturday" }
    whan 7 -> { blether "Sunday" }
}
```

## Wildcard Pattern

Use `_` to match anything (default case):

```scots
ken score = 85

keek score {
    whan 100 -> { blether "Perfect!" }
    whan 90 -> { blether "Excellent!" }
    whan 80 -> { blether "Great!" }
    whan _ -> { blether "Keep trying!" }
}
```

## Variable Binding

Capture the matched value in a variable:

```scots
ken mystery = 42

keek mystery {
    whan 0 -> { blether "Zero" }
    whan x -> { blether f"Got: {x}" }
}
# Output: "Got: 42"
```

## Match in Functions

Pattern matching works great in functions:

```scots
dae describe_number(n) {
    keek n {
        whan 0 -> { gie "zero" }
        whan 1 -> { gie "one" }
        whan 2 -> { gie "two" }
        whan _ -> { gie "many" }
    }
}

blether describe_number(0)   # "zero"
blether describe_number(1)   # "one"
blether describe_number(42)  # "many"
```

## Matching Strings

```scots
ken command = "quit"

keek command {
    whan "start" -> { blether "Starting..." }
    whan "stop" -> { blether "Stopping..." }
    whan "quit" -> { blether "Goodbye!" }
    whan "exit" -> { blether "Goodbye!" }
    whan _ -> { blether f"Unknown command: {command}" }
}
```

## Match with Return Values

```scots
dae grade_to_points(grade) {
    keek grade {
        whan "A" -> { gie 4 }
        whan "B" -> { gie 3 }
        whan "C" -> { gie 2 }
        whan "D" -> { gie 1 }
        whan "F" -> { gie 0 }
        whan _ -> { gie -1 }
    }
}

blether grade_to_points("A")  # 4
blether grade_to_points("C")  # 2
blether grade_to_points("X")  # -1
```

## Practical Examples

### Command Parser

```scots
dae handle_command(cmd) {
    ken parts = split(cmd, " ")
    ken action = heid(parts)

    keek action {
        whan "help" -> {
            blether "Available commands: help, add, remove, list"
        }
        whan "add" -> {
            gin len(parts) < 2 {
                blether "Usage: add <item>"
            } ither {
                blether f"Adding: {parts[1]}"
            }
        }
        whan "remove" -> {
            gin len(parts) < 2 {
                blether "Usage: remove <item>"
            } ither {
                blether f"Removing: {parts[1]}"
            }
        }
        whan "list" -> {
            blether "Listing all items..."
        }
        whan _ -> {
            blether f"Unknown command: {action}"
        }
    }
}

handle_command("help")
handle_command("add task")
handle_command("remove task")
handle_command("unknown")
```

### State Machine

```scots
ken state = "idle"

dae process_event(event) {
    keek state {
        whan "idle" -> {
            keek event {
                whan "start" -> {
                    state = "running"
                    blether "Started!"
                }
                whan _ -> { blether "Ignoring in idle state" }
            }
        }
        whan "running" -> {
            keek event {
                whan "pause" -> {
                    state = "paused"
                    blether "Paused"
                }
                whan "stop" -> {
                    state = "idle"
                    blether "Stopped"
                }
                whan _ -> { blether "Ignoring in running state" }
            }
        }
        whan "paused" -> {
            keek event {
                whan "resume" -> {
                    state = "running"
                    blether "Resumed"
                }
                whan "stop" -> {
                    state = "idle"
                    blether "Stopped"
                }
                whan _ -> { blether "Ignoring in paused state" }
            }
        }
    }
}

process_event("start")   # Started!
process_event("pause")   # Paused
process_event("resume")  # Resumed
process_event("stop")    # Stopped
```

### Type Dispatcher

```scots
dae describe_value(value) {
    ken type = whit_kind(value)

    keek type {
        whan "integer" -> { gie f"An integer: {value}" }
        whan "float" -> { gie f"A float: {value}" }
        whan "string" -> { gie f"A string of length {len(value)}" }
        whan "list" -> { gie f"A list with {len(value)} items" }
        whan "dict" -> { gie f"A dict with {len(keys(value))} keys" }
        whan "boolean" -> { gie f"A boolean: {value}" }
        whan "null" -> { gie "Nothing (null)" }
        whan _ -> { gie f"Unknown type: {type}" }
    }
}

blether describe_value(42)
blether describe_value(3.14)
blether describe_value("hello")
blether describe_value([1, 2, 3])
blether describe_value({"a": 1})
```

### Menu System

```scots
dae show_menu() {
    blether "=== Main Menu ==="
    blether "1. New Game"
    blether "2. Load Game"
    blether "3. Settings"
    blether "4. Quit"
    blether "================="
}

dae handle_menu_choice(choice) {
    keek choice {
        whan "1" -> {
            blether "Starting new game..."
            gie aye
        }
        whan "2" -> {
            blether "Loading game..."
            gie aye
        }
        whan "3" -> {
            blether "Opening settings..."
            gie aye
        }
        whan "4" -> {
            blether "Goodbye!"
            gie nae
        }
        whan _ -> {
            blether "Invalid choice, try again."
            gie aye
        }
    }
}

ken running = aye
whiles running {
    show_menu()
    ken choice = speir "Enter choice: "
    running = handle_menu_choice(choice)
}
```

### HTTP Status Codes

```scots
dae describe_status(code) {
    keek code {
        whan 200 -> { gie "OK - Request succeeded" }
        whan 201 -> { gie "Created - Resource created" }
        whan 204 -> { gie "No Content - Request succeeded, no body" }
        whan 301 -> { gie "Moved Permanently - Resource relocated" }
        whan 302 -> { gie "Found - Temporary redirect" }
        whan 400 -> { gie "Bad Request - Invalid request" }
        whan 401 -> { gie "Unauthorized - Authentication required" }
        whan 403 -> { gie "Forbidden - Access denied" }
        whan 404 -> { gie "Not Found - Resource doesn't exist" }
        whan 500 -> { gie "Internal Server Error - Server problem" }
        whan 502 -> { gie "Bad Gateway - Invalid upstream response" }
        whan 503 -> { gie "Service Unavailable - Server overloaded" }
        whan _ -> { gie f"Unknown status code: {code}" }
    }
}

blether describe_status(200)  # "OK - Request succeeded"
blether describe_status(404)  # "Not Found - Resource doesn't exist"
blether describe_status(418)  # "Unknown status code: 418"
```

### Calculator Operations

```scots
dae calculate(a, op, b) {
    keek op {
        whan "+" -> { gie a + b }
        whan "-" -> { gie a - b }
        whan "*" -> { gie a * b }
        whan "/" -> {
            gin b == 0 {
                gie naething
            }
            gie a / b
        }
        whan "%" -> { gie a % b }
        whan "^" -> { gie pooer(a, b) }
        whan _ -> {
            blether f"Unknown operator: {op}"
            gie naething
        }
    }
}

blether calculate(10, "+", 5)  # 15
blether calculate(10, "-", 3)  # 7
blether calculate(10, "*", 4)  # 40
blether calculate(10, "/", 2)  # 5
blether calculate(2, "^", 8)   # 256
```

## Pattern Matching vs If/Else

Pattern matching often leads to cleaner code:

```scots
# Using gin/ither (verbose)
dae day_name_if(n) {
    gin n == 1 {
        gie "Monday"
    } ither gin n == 2 {
        gie "Tuesday"
    } ither gin n == 3 {
        gie "Wednesday"
    } ither gin n == 4 {
        gie "Thursday"
    } ither gin n == 5 {
        gie "Friday"
    } ither gin n == 6 {
        gie "Saturday"
    } ither gin n == 7 {
        gie "Sunday"
    } ither {
        gie "Unknown"
    }
}

# Using keek/whan (cleaner)
dae day_name(n) {
    keek n {
        whan 1 -> { gie "Monday" }
        whan 2 -> { gie "Tuesday" }
        whan 3 -> { gie "Wednesday" }
        whan 4 -> { gie "Thursday" }
        whan 5 -> { gie "Friday" }
        whan 6 -> { gie "Saturday" }
        whan 7 -> { gie "Sunday" }
        whan _ -> { gie "Unknown" }
    }
}
```

## Exercises

1. **Roman Numerals**: Convert a single digit (1-9) to Roman numerals

2. **Traffic Light**: Create a traffic light state machine

3. **Rock Paper Scissors**: Implement the game logic with pattern matching

<details>
<summary>Solutions</summary>

```scots
# 1. Roman Numerals
dae to_roman(n) {
    keek n {
        whan 1 -> { gie "I" }
        whan 2 -> { gie "II" }
        whan 3 -> { gie "III" }
        whan 4 -> { gie "IV" }
        whan 5 -> { gie "V" }
        whan 6 -> { gie "VI" }
        whan 7 -> { gie "VII" }
        whan 8 -> { gie "VIII" }
        whan 9 -> { gie "IX" }
        whan _ -> { gie "?" }
    }
}

fer i in 1..10 {
    blether f"{i} = {to_roman(i)}"
}

# 2. Traffic Light
kin TrafficLight {
    dae init() {
        masel.state = "red"
    }

    dae next() {
        keek masel.state {
            whan "red" -> { masel.state = "green" }
            whan "green" -> { masel.state = "yellow" }
            whan "yellow" -> { masel.state = "red" }
        }
    }

    dae current() {
        keek masel.state {
            whan "red" -> { gie "STOP" }
            whan "yellow" -> { gie "CAUTION" }
            whan "green" -> { gie "GO" }
        }
    }
}

ken light = TrafficLight()
fer i in 1..7 {
    blether f"Light: {light.state} - {light.current()}"
    light.next()
}

# 3. Rock Paper Scissors
dae rps_winner(player1, player2) {
    gin player1 == player2 {
        gie "tie"
    }

    keek player1 {
        whan "rock" -> {
            keek player2 {
                whan "scissors" -> { gie "player1" }
                whan "paper" -> { gie "player2" }
            }
        }
        whan "paper" -> {
            keek player2 {
                whan "rock" -> { gie "player1" }
                whan "scissors" -> { gie "player2" }
            }
        }
        whan "scissors" -> {
            keek player2 {
                whan "paper" -> { gie "player1" }
                whan "rock" -> { gie "player2" }
            }
        }
    }
    gie "invalid"
}

blether rps_winner("rock", "scissors")  # "player1"
blether rps_winner("paper", "rock")     # "player1"
blether rps_winner("rock", "paper")     # "player2"
blether rps_winner("rock", "rock")      # "tie"
```

</details>

## Next Steps

Learn about [functional programming](./10-functional.md) with higher-order functions.
