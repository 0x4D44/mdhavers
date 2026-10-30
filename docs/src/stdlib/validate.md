# validate

Small validators for strings, numbers and lists, plus a chainable `Schema` class that checks the fields of a dict and collects readable error messages.

Import: `fetch "stdlib/validate"`

Backend: pure mdhavers (uses `ord`, `floor`, `starts_wi`, `whit_kind`, `contains`, `keys`). Loading the module prints `Validation module loaded! Ready tae check yer data!`. Nothing here uses `hurl`; validators return `aye` or `nae`, and `Schema` collects its messages in a list. Helpers starting with `_` are internal and not documented (there are none in this file).

## Functions

**String validators**

### is_blank(s)
Empty or only spaces, tabs, newlines and carriage returns.

### is_present(s)
`nae is_blank(s)`.

### min_length(s, min)
`len(s) >= min`.

### max_length(s, max)
`len(s) <= max`.

### length_between(s, min, max)
`min <= len(s) <= max`.

### is_alpha(s)
Non-empty and ASCII letters only.

### is_numeric(s)
Non-empty and ASCII digits only.

### is_alphanumeric(s)
Non-empty and ASCII letters or digits only.

### is_email(s)
Rough check: at least 5 characters, exactly one `@` that is neither first nor last, and a `.` somewhere after it.

### is_url(s)
Starts with `http://` or `https://`.

**Number validators**

### is_positive(n)
`n > 0`.

### is_negative(n)
`n < 0`.

### is_zero(n)
`n == 0`.

### in_range(n, min, max)
Inclusive range test.

### is_integer(n)
`floor(n) == n`, so `2.0` counts as an integer.

### is_even(n)
`n % 2 == 0`.

### is_odd(n)
`n % 2 != 0`.

**List validators**

### is_empty(lst)
`len(lst) == 0`.

### is_nae_empty(lst)
`len(lst) > 0`.

### min_size(lst, min)
`len(lst) >= min`.

### max_size(lst, max)
`len(lst) <= max`.

### all_pass(lst, predicate)
Every item passes (`aye` for an empty list).

### any_pass(lst, predicate)
At least one item passes.

### none_pass(lst, predicate)
No item passes.

**Schema builders**

### user_registration_schema()
A ready `Schema`: `username` (required, string, 3 to 20 characters), `email` (required, valid email) and `age` (required, number, 13 to 120).

### product_schema()
A ready `Schema`: `name` (required, string, at least 1 character) and `price` (required, number, at least 0).

## Classes

### Schema
`Schema()` takes no arguments. Fields: `rules` (dict of field name to list of rule dicts) and `errors` (list of strings). Each rule method returns `masel`, so calls can be chained, and several rules can be added to one field.

| Method | Rule added |
| --- | --- |
| `required(field_name)` | Value must be present and, for strings, not blank. |
| `string(field_name)` | If present, must be a string. |
| `number(field_name)` | If present, must be an integer or float. |
| `min_len(field_name, min)` | Strings must have at least `min` characters. |
| `max_len(field_name, max)` | Strings must have at most `max` characters. |
| `min_val(field_name, min)` | If present, value must be at least `min`. |
| `max_val(field_name, max)` | If present, value must be at most `max`. |
| `email(field_name)` | Strings must pass `is_email`. |
| `validate(data)` | Clears old errors, checks every rule against the dict `data` and returns `aye` if there were no errors. |
| `get_errors()` | List of error strings from the last `validate`. |
| `is_valid()` | `aye` if the last `validate` found no errors. |

Messages look like `username must be at least 3 characters`, `email must be a valid email`, `age must be at most 120`, `name is required`, `x must be a string`, `x must be a number`. Rules other than `required` skip fields that are missing, so a missing field is only reported when it is also marked `required`. `min_val` and `max_val` do not check the type first: if the field holds a non-number such as a string, `validate` fails with a runtime type error (`Cannae compare string wi' integer`) instead of adding a message, even if `number` was also added (`validate.braw:344`, `:350`).

## Compared with strings and text

`is_blank`, `is_alpha`, `is_numeric` and `is_alphanumeric` also exist in [text](./text.md), and `is_positive`, `is_negative`, `is_even` and `is_odd` in [prelude](./prelude.md) (the latter two as built-ins). They behave the same for normal input. `validate` is the one to use for string-length and data checks and the only one with `Schema`; do not fetch it together with [text](./text.md) unless you accept that the later module's definitions win.

## Example

```mdhavers
fetch "stdlib/validate"

ken schema = Schema()
schema.required("name").string("name").min_len("name", 3)
schema.required("age").number("age").min_val("age", 18)

blether schema.validate({"name": "Jo", "age": 16})
blether schema.get_errors()
blether schema.validate({"name": "Jock", "age": 30})
blether is_email("jock@example.com")
```
