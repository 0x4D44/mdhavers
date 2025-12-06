# Error Handling

Handle errors gracefully with `hae_a_bash` and `gin_it_gangs_wrang`.

## Try/Catch Basics

Use `hae_a_bash` (have a bash/try) and `gin_it_gangs_wrang` (if it goes wrong/catch):

```scots
hae_a_bash {
    # Risky code here
    ken result = 10 / 0
    blether result
} gin_it_gangs_wrang err {
    blether f"Caught error: {err}"
}
```

Output:
```
Caught error: Division by zero
```

## Common Error Types

### Division by Zero

```scots
hae_a_bash {
    ken x = 5 / 0
} gin_it_gangs_wrang err {
    blether f"Math error: {err}"
}
```

### Undefined Variables

```scots
hae_a_bash {
    blether undefined_variable
} gin_it_gangs_wrang err {
    blether f"Variable error: {err}"
}
```

### Index Out of Bounds

```scots
hae_a_bash {
    ken list = [1, 2, 3]
    blether list[100]
} gin_it_gangs_wrang err {
    blether f"Index error: {err}"
}
```

### Type Errors

```scots
hae_a_bash {
    ken result = "hello" + 5
} gin_it_gangs_wrang err {
    blether f"Type error: {err}"
}
```

### Key Not Found

```scots
hae_a_bash {
    ken dict = {"name": "Hamish"}
    blether dict["age"]
} gin_it_gangs_wrang err {
    blether f"Key error: {err}"
}
```

## Error Recovery

The program continues after catching an error:

```scots
ken numbers = [1, 2, 0, 4, 5]

fer n in numbers {
    hae_a_bash {
        ken result = 100 / n
        blether f"100 / {n} = {result}"
    } gin_it_gangs_wrang err {
        blether f"Skipping {n}: {err}"
    }
}
blether "Done processing all numbers!"
```

Output:
```
100 / 1 = 100
100 / 2 = 50
Skipping 0: Division by zero
100 / 4 = 25
100 / 5 = 20
Done processing all numbers!
```

## Nested Error Handling

Try/catch blocks can be nested:

```scots
hae_a_bash {
    blether "Outer try block"

    hae_a_bash {
        blether "Inner try block"
        ken trouble = 1 / 0
    } gin_it_gangs_wrang inner_err {
        blether f"Inner catch: {inner_err}"
    }

    blether "Continuing in outer block"
} gin_it_gangs_wrang outer_err {
    blether f"Outer catch: {outer_err}"
}
```

Output:
```
Outer try block
Inner try block
Inner catch: Division by zero
Continuing in outer block
```

## Error Handling in Functions

### Returning Error Status

```scots
dae safe_divide(a, b) {
    hae_a_bash {
        gie {"success": aye, "value": a / b}
    } gin_it_gangs_wrang err {
        gie {"success": nae, "error": err}
    }
}

ken result = safe_divide(10, 2)
gin result["success"] {
    blether f"Result: {result['value']}"
} ither {
    blether f"Error: {result['error']}"
}

ken bad_result = safe_divide(10, 0)
gin bad_result["success"] {
    blether f"Result: {bad_result['value']}"
} ither {
    blether f"Error: {bad_result['error']}"
}
```

### Early Return on Error

```scots
dae process_data(data) {
    ken result = naething

    hae_a_bash {
        result = data / 2
    } gin_it_gangs_wrang err {
        blether f"Failed to process: {err}"
        gie naething
    }

    gie result
}

ken good = process_data(10)
blether good  # 5

ken bad = process_data("not a number")
blether bad   # naething
```

## Validation with mak_siccar

Use `mak_siccar` (make sure - famously said by Robert the Bruce!) for assertions:

```scots
dae calculate_age(birth_year, current_year) {
    mak_siccar birth_year > 0, "Birth year must be positive"
    mak_siccar current_year >= birth_year, "Current year cannae be before birth"

    gie current_year - birth_year
}

# This works
ken age = calculate_age(1990, 2024)
blether f"Age: {age}"

# This fails with assertion
hae_a_bash {
    ken bad_age = calculate_age(-100, 2024)
} gin_it_gangs_wrang err {
    blether f"Assertion failed: {err}"
}
```

## Practical Patterns

### Safe File Reading

```scots
dae read_config(path) {
    hae_a_bash {
        gin nae file_exists(path) {
            gie {"error": "File not found"}
        }
        ken content = read_file(path)
        gie {"success": aye, "content": content}
    } gin_it_gangs_wrang err {
        gie {"error": tae_string(err)}
    }
}

ken config = read_config("config.txt")
gin dict_has(config, "error") {
    blether f"Could not read config: {config['error']}"
} ither {
    blether f"Config loaded: {config['content']}"
}
```

### Input Validation

```scots
dae get_positive_number() {
    ken attempts = 0
    ken max_attempts = 3

    whiles attempts < max_attempts {
        ken input = speir "Enter a positive number: "

        hae_a_bash {
            ken num = tae_int(input)
            gin num > 0 {
                gie num
            }
            blether "Number must be positive!"
        } gin_it_gangs_wrang err {
            blether "That's nae a valid number!"
        }

        attempts = attempts + 1
    }

    blether "Too many failed attempts."
    gie naething
}

ken number = get_positive_number()
gin number != naething {
    blether f"You entered: {number}"
}
```

### Batch Processing with Error Collection

```scots
dae process_items(items) {
    ken results = []
    ken errors = []

    fer item in items {
        hae_a_bash {
            # Process the item
            ken processed = item * 2
            shove(results, {"item": item, "result": processed})
        } gin_it_gangs_wrang err {
            shove(errors, {"item": item, "error": tae_string(err)})
        }
    }

    gie {"results": results, "errors": errors}
}

ken items = [1, 2, "bad", 4, naething]
ken outcome = process_items(items)

blether f"Processed: {len(outcome['results'])} items"
blether f"Errors: {len(outcome['errors'])} items"

fer error in outcome["errors"] {
    blether f"  Failed on {error['item']}: {error['error']}"
}
```

### Default Values on Error

```scots
dae safe_get(dict, key, default) {
    hae_a_bash {
        gie dict[key]
    } gin_it_gangs_wrang _ {
        gie default
    }
}

ken person = {"name": "Hamish"}

blether safe_get(person, "name", "Unknown")  # "Hamish"
blether safe_get(person, "age", 0)           # 0
blether safe_get(person, "city", "N/A")      # "N/A"
```

## Error Messages in Scots

mdhavers gives you error messages in Scots dialect:

| Error | Scots Message |
|-------|---------------|
| Undefined variable | "Och! Ah dinnae ken whit 'x' is" |
| Division by zero | "Ye numpty! Tryin' tae divide by zero" |
| Type error | "That's pure mince! Type error" |
| Wrong arguments | "Yer bum's oot the windae!" |
| Syntax error | "Haud yer wheesht! Unexpected token" |
| Assertion failed | "Mak siccar failed!" |

## Exercises

1. **Safe Calculator**: Create a calculator function that handles all errors gracefully

2. **Retry Mechanism**: Create a function that retries an operation up to N times

3. **Result Type**: Create a Result class for handling success/failure

<details>
<summary>Solutions</summary>

```scots
# 1. Safe Calculator
dae calculate(a, op, b) {
    hae_a_bash {
        keek op {
            whan "+" -> { gie a + b }
            whan "-" -> { gie a - b }
            whan "*" -> { gie a * b }
            whan "/" -> {
                mak_siccar b != 0, "Cannot divide by zero"
                gie a / b
            }
            whan _ -> { gie {"error": "Unknown operator"} }
        }
    } gin_it_gangs_wrang err {
        gie {"error": tae_string(err)}
    }
}

blether calculate(10, "+", 5)   # 15
blether calculate(10, "/", 0)   # {"error": "..."}
blether calculate(10, "^", 2)   # {"error": "Unknown operator"}

# 2. Retry Mechanism
dae retry(operation, max_attempts) {
    ken attempts = 0
    ken last_error = naething

    whiles attempts < max_attempts {
        hae_a_bash {
            ken result = operation()
            gie {"success": aye, "value": result}
        } gin_it_gangs_wrang err {
            last_error = err
            attempts = attempts + 1
            blether f"Attempt {attempts} failed: {err}"
        }
    }

    gie {"success": nae, "error": last_error, "attempts": attempts}
}

# Example usage
ken flaky_count = 0
dae flaky_operation() {
    # Simulate an operation that sometimes fails
    ken random = jammy(1, 10)
    gin random < 7 {
        blether "Operation failed randomly"
        ken x = 1 / 0  # Force an error
    }
    gie "Success!"
}

# ken result = retry(flaky_operation, 5)

# 3. Result Type
kin Result {
    dae init(success, value, error) {
        masel.success = success
        masel.value = value
        masel.error = error
    }

    dae is_ok() {
        gie masel.success
    }

    dae unwrap() {
        gin nae masel.success {
            blether f"Unwrap failed: {masel.error}"
        }
        gie masel.value
    }

    dae unwrap_or(default) {
        gin masel.success {
            gie masel.value
        }
        gie default
    }
}

dae ok(value) {
    gie Result(aye, value, naething)
}

dae err(message) {
    gie Result(nae, naething, message)
}

# Usage
dae parse_int(s) {
    hae_a_bash {
        gie ok(tae_int(s))
    } gin_it_gangs_wrang e {
        gie err(f"Failed to parse '{s}'")
    }
}

ken r1 = parse_int("42")
blether r1.unwrap()        # 42

ken r2 = parse_int("abc")
blether r2.unwrap_or(0)    # 0
```

</details>

## Next Steps

Learn about [pattern matching](./09-pattern-matching.md) for elegant control flow.
