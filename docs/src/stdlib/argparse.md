# argparse

Command-line argument parsing: an `ArgParser` collects `Argument` definitions (options, flags and positionals), parses a list of strings into a dict, prints help, and a few `key=value` helpers cover simple cases. Parse failures print a message and return `naething` instead of hurling.

Import: `fetch "stdlib/argparse"`

Backend note: pure mdhavers, no native built-ins. Loading prints `Argparse module loaded! Ready tae parse yer arguments!`. Helpers starting with `_` (`_find_arg`, `_convert_value` on `ArgParser`) are internal and not documented.

## Constants

`ARG_STRING` (`"string"`), `ARG_INT` (`"int"`), `ARG_FLOAT` (`"float"`), `ARG_BOOL` (`"bool"`), `ARG_LIST` (`"list"`). `ARG_LIST` is accepted as a type name but values are not split; they stay strings.

## Functions

### arg(name)
Returns `Argument(name)`. A name starting with `-` (such as `"--count"`) is an option; any other name is a positional argument.

### make_parser(name)
Returns `ArgParser(name)`.

### parse_simple(args)
Takes a list of strings. `key=value` becomes `result["key"] = "value"` (split at the first `=`); anything else becomes `result[arg] = aye`.

### has_flag(args, flag)
`aye` if any element equals `flag`, `--flag` or `-flag`; otherwise `nae`.

### get_value(args, key, default = naething)
Looks for `key`, `--key` or `-key` followed by a value, or `key=value` / `--key=value`. Returns the value, else `default`. A matching option with no following value falls through to later elements and finally `default`.

## Classes

### Argument
`Argument(name)`. Defaults: `short` naething, `arg_type` `"string"`, `default` naething, `required` nae, `help` `""`, `choices` naething, `is_flag` nae. All builder methods return the argument so they chain.

- `short_name(short)`: set the short form, such as `"-n"`.
- `type(arg_type)`: one of the `ARG_*` constants.
- `default_value(value)`: value used when the option is absent.
- `make_required()`: parsing fails if the value is still naething.
- `help_text(text)`: text shown by help.
- `valid_choices(choices)`: list of allowed (converted) values.
- `flag()`: boolean flag taking no value; type becomes bool, default `nae`, set to `aye` when present.

### ArgParser
`ArgParser(name)`. Fields: `name`, `description`, `version` (`"1.0.0"`), `arguments` (options), `positional`.

- `describe(desc)`, `with_version(ver)`: chainable setters.
- `add(argument)`: files it under options or positionals by whether its name starts with `-`; returns the parser.
- `parse(args)`: `args` is a list of strings (without the program name). Returns a dict keyed by the option name with leading dashes removed (`--count` gives `count`; positionals keep their name). Defaults are filled first. Values are converted by type (`int` via `tae_int`, `float` via `tae_float`, bool true for `true`, `aye`, `1`, `yes`). An option may be given by its name or its short form. Extra positionals are ignored. It prints a message and returns `naething` in these cases (message text exactly as printed):
  - `--help` or `-h`: prints the help (see `show_help`).
  - `--version` or `-v`: prints `<name> version <version>`.
  - `Unknown option: <arg>`
  - `Option <arg> requires a value`
  - `Invalid value fer <arg>. Must be one o': <choices>`
  - `Required option missing: <name>`
  - `Required argument missing: <name>`
- `show_help()`: prints usage, description, positional arguments and options with type, default and required hints. `-h` and `-v` are reserved; do not use them as short names.

Because `parse` returns `naething` both for help/version and for errors, check the result before indexing it. A non-numeric value for an int option makes `tae_int` hurl its own error (`Cannae turn '<value>' intae an integer`).

## Example

```mdhavers
fetch "stdlib/argparse"

ken p = ArgParser("tool").describe("Does things")
p.add(arg("--name").short_name("-n").default_value("anon"))
p.add(arg("--count").type(ARG_INT).make_required())
p.add(arg("--verbose").short_name("-V").flag())
p.add(arg("file").make_required())

ken opts = p.parse(["--count", "3", "-V", "in.txt", "-n", "Bob"])
blether opts["name"]
blether opts["count"] + 1
blether opts["verbose"]
blether opts["file"]
blether parse_simple(["a=1", "flag"])
```
