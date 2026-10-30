# config

Configuration holders: a key-value `Config` with defaults and typed getters, an INI parser, an environment-style `NAME=value` parser and a set of feature flags. All operate on strings you supply; none read files or the real environment (read the file yourself and pass its text in).

Import: `fetch "stdlib/config"`

Pure mdhavers; works wherever the interpreter does. Loading the module prints `Config module loaded! Ready tae set things up!`. This module hurls nothing itself.

## Functions

### parse_ini(content)
Creates an `IniConfig`, parses `content` and returns it.

### parse_env(content)
Creates an `EnvConfig`, parses `content` and returns it.

### config_from_dict(data)
A `Config` with every pair of `data` set.

## Classes

### Config
`Config()`. Fields `data`, `defaults`. Setters return the config.

- `set(key, value)`, `set_default(key, value)`, `set_defaults(dict)`.
- `get(key, default = naething)`: value, else the default registered with `set_default`, else `default`.
- `has(key)`: in data or defaults.
- `delete(key)`: if the key is in the data, sets it to `naething` (it cannot truly remove it); a key that only has a registered default is left alone. Afterwards `has(key)` is still `aye` and `get(key, d)` returns `naething`, not `d` or the registered default; `all_keys` skips it, and `tae_dict` skips the `naething` value (so a registered default shows through again).
- `all_keys()`: keys with non-`naething` values, then default-only keys.
- `tae_dict()`: defaults overlaid by data.
- `merge(other_config)`: copies `other_config.tae_dict()` into data.
- `get_int(key, default = 0)`, `get_float(key, default = 0.0)`: convert with `tae_int` / `tae_float`; missing gives `default`.
- `get_bool(key, default = nae)`: `aye` for `"true"`, `"aye"`, `"yes"`, `"1"` or the boolean `aye` (exact case); anything else present gives `nae`.
- `get_list(key, separator = ",")`: split string, `[]` if missing or empty.

### IniConfig
`IniConfig()`. Fields `sections` (dict of dicts) and `current_section`; keys before any header go in section `"default"`.

- `parse(content)`: skips blank lines and lines starting with `#` or `;`; `[name]` starts a section; `key = value` pairs are trimmed and one pair of surrounding single or double quotes is removed. Values are strings. Returns the config.
- `get(section, key, default = naething)`, `get_default(key, default = naething)`.
- `set(section, key, value)`: returns the config.
- `section_names()`, `section_keys(section)` (`[]` if unknown), `section(section)` (dict, `{}` if unknown), `has_section(section)`.
- `tae_string()`: INI text (the `default` section has no header). Call it as a method.

### EnvConfig
`EnvConfig()`. Names are upper-cased on set, get and has.

- `set(name, value)`: stores `tae_string(value)`; returns the config.
- `get(name, default = "")`.
- `parse(content)`: `NAME=value` lines, skipping blanks and `#` comments, with the same quote removal as INI. Returns the config.
- `tae_dict()`: the variables dict.
- `has(name)`.
- `get_int(name, default = 0)`: `default` when missing or empty.
- `get_bool(name, default = nae)`: `aye` for `true`, `1`, `yes`, `aye` (case-insensitive).

### FeatureFlags
`FeatureFlags()`. Field `flags`. Mutators return the object.

- `enable(name)`, `disable(name)`, `toggle(name)` (an unknown flag becomes enabled).
- `is_enabled(name)`: `nae` for unknown flags.
- `enabled_features()`: names whose value is truthy.
- `set_flags(dict)`: bulk set.

## Example

```mdhavers
fetch "stdlib/config"

ken ini = parse_ini("name = demo\n[server]\nport = 8080\n")
blether ini.get_default("name")
blether ini.get("server", "port")

ken cfg = Config()
cfg.set_default("level", "info").set("port", "80")
blether cfg.get_int("port")
blether cfg.get("level")
```
