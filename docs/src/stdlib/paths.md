# paths

Pure-string path helpers using `/` as the separator (no Windows drive handling), a `Path` class that adds file reading and writing, file-type checks by extension and a `PathBuilder`. Nothing here checks the filesystem except the `Path` methods that read or write.

Import: `fetch "stdlib/paths"`

The string helpers are pure mdhavers. `Path.exists`, `read`, `read_lines`, `write` and `append` use the file built-ins (`file_exists`, `read_file`, `read_lines`, `scrieve`, `append_file`), so they need a native/interpreter build with file access. Loading the module prints `Paths module loaded! Ready tae find yer way aboot!`. This module hurls nothing itself.

## Constants

`PATH_SEP` (`"/"`), `EXT_SEP` (`"."`), `COMMON_EXTENSIONS` (dict from extension to a description such as `"Rust source"`).

## Functions

### basename(path)
Last component, ignoring trailing slashes. `""` gives `""`.

### dirname(path)
All but the last component: `"."` if none, `"/"` for a top-level absolute path, `"."` for `""`.

### extension(path)
Extension without the dot, or `""` (including hidden files such as `.bashrc`). `x.tar.gz` gives `gz`.

### stem(path)
Base name without its last extension (`x.tar.gz` gives `x.tar`).

### path_join(parts)
Joins a list of parts with one `/` between. A part that starts with `/` is appended after the previous one (it does not reset to the root), and empty parts are skipped.

### join_path(a, b)
`path_join([a, b])`.

### is_absolute(path)
`aye` if non-empty and starts with `/`.

### is_relative(path)
Opposite of `is_absolute`.

### normalize(path)
Resolves `.`, `..` and repeated slashes lexically. `""` gives `"."`; leading `..` are kept for relative paths and dropped above the root for absolute ones.

### path_parts(path)
List of non-empty components.

### path(str)
Returns a `Path`.

### file_type(path)
Description from `COMMON_EXTENSIONS` (extension lower-cased), else `"<ext> file"`, else `"unknown file"`.

### is_source_file(path)
`aye` for braw, rs, py, js, ts, go, java, c, cpp, rb, h.

### is_image(path)
`aye` for png, jpg, jpeg, gif, svg, bmp, webp.

### is_document(path)
`aye` for md, txt, pdf, doc, docx, rtf.

### is_config(path)
`aye` for ini, cfg, conf, env, toml, yaml, yml, json.

## Classes

### Path
`Path(path_str = ".")`. Field `path`. Methods returning paths return new `Path`s.

- `tae_string()`: the string. Call it as a method; the global `tae_string(p)` prints `<Path instance>`.
- `basename()`, `extension()`, `stem()`, `parts()`, `is_absolute()`, `is_relative()`: as the functions.
- `dirname()` and `parent()`: `Path` of the directory.
- `join(other)`: `other` is a string or a `Path`.
- `normalize()`: normalised `Path`.
- `exists()`: `file_exists`.
- `read()`, `read_lines()`: file contents or list of lines.
- `write(content)`, `append(content)`: write or append; return the path.
- `with_extension(ext)`: swaps the extension (`""` removes it). A bare name gets a `./` prefix (`Path("rel.txt").with_extension("md")` is `./rel.md`).
- `with_filename(name)`: swaps the last component (same `./` prefix for a bare name).

### PathBuilder
`PathBuilder(base = ".")`. Field `parts`.

- `add(component)`, `add_all(components)`: return the builder.
- `build()`: a `Path`.
- `tae_string()`: the joined string.

## Example

```mdhavers
fetch "stdlib/paths"

blether basename("/a/b.txt")
blether normalize("/a/./b/../c")
ken p = path("/tmp/report.txt")
blether p.with_extension("md").tae_string()
blether PathBuilder("a").add("b").add_all(["c", "d"]).tae_string()
```
