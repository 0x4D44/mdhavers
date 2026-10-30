# table

Tabular data for mdhavers: a `Table` class holding rows as dicts with named columns, with filtering, sorting, aggregates, grouping, joins, pivoting and rendering as an ASCII grid or CSV. Builders create tables from lists of lists or dicts, or from a CSV string.

Import: `fetch "stdlib/table"`

Backend note: pure mdhavers, no native built-ins. Loading prints `Table module loaded! Ready tae wrangle yer data!`. Helpers starting with `_` are internal and not documented (none here). No function hurls its own error; a bad column name in `sort_by`, `distinct` and the joins fails with the interpreter's ordinary error.

## Functions

### pad_right(str, width)
Returns `str` (converted with `tae_string`) padded on the right with spaces to `width`.

### pad_left(str, width)
Same, padding on the left.

### repeat(str, times)
Returns `str` repeated `times` times. Shadows the built-in `repeat` of the same name if one is in scope.

### make_table(columns)
Returns `Table(columns)`.

### table_from_list(data, columns = naething)
Builds a table from a list of rows. For an empty list returns `Table([])`. When `columns` is omitted, a list of dicts takes the keys of the first row; a list of lists names the columns `col0`, `col1`, ...

### table_from_csv(csv_string)
Splits on newlines and commas, first line is the header. Cells are trimmed and stay strings (no number conversion). Blank lines are skipped. Quoted fields with commas are not supported.

### inner_join(table1, table2, on_column)
New table with the columns of `table1` followed by the extra columns of `table2`; one output row for each pair of rows with equal, non-naething `on_column` values.

### left_join(table1, table2, on_column)
Like `inner_join` but keeps every `table1` row; unmatched rows get `naething` in the `table2` columns (rendered as `naething`).

### concat_tables(tables)
Stacks the rows of a list of tables under the columns of the first. Empty list gives `Table([])`.

### pivot(table, index_col, pivot_col, value_col)
Groups rows by `index_col`; each distinct `pivot_col` value becomes a column holding the sum of `value_col` (0 where absent). Values must be numbers. Group keys are strings (via `group_by`).

## Classes

### Table
`Table(columns)` where `columns` is a list of column names. Fields: `columns`, `rows` (list of dicts), `column_widths` (dict of column name to display width). Query methods (`filter`, `map_rows`, `sort_by`, `select`, `distinct`, `take`, `skip`) return new tables. Rows are not copied, so a derived table shares row dicts with its source (a `set` on one is seen by the other).

- `add_row(row)`: row is a list (matched to columns by position; short lists fill with `naething`) or a dict. Returns the table.
- `add_rows(rows)`: adds each; returns the table.
- `get_row(index)`: the row dict, or `naething` if out of range.
- `get_column(col_name)`: list of the column's values (`naething` where a row lacks it).
- `get(row_idx, col_name)`: one value or `naething`.
- `set(row_idx, col_name, value)`: updates in place when the index is valid; returns the table.
- `filter(predicate)`: new table of rows where `predicate(row)` is truthy.
- `map_rows(transform)`: new table of `transform(row)` results.
- `sort_by(col_name, ascending = aye)`: new table, bubble sort (O(n squared)); equal keys keep their order.
- `select(col_names)`: new table with only those columns.
- `sum(col_name)`, `avg(col_name)`, `min_val(col_name)`, `max_val(col_name)`: aggregates that skip `naething` cells. `avg` of nothing is 0, `min_val`/`max_val` of nothing is `naething`. `avg` of integers uses integer division.
- `count_rows()`: number of rows.
- `group_by(col_name)`: a dict from the value (as a string) to the list of rows; rows without the column go under `"naething"`.
- `distinct(col_names = naething)`: new table with duplicates removed, comparing all columns or just `col_names`; first occurrence wins.
- `take(n)`, `skip(n)`: first `n` rows, or all but the first `n`.
- `to_string()`: the ASCII grid (`+---+`, `| a |`) as one string.
- `display()`: prints `to_string()`, returns the table.
- `to_csv()`: header plus rows, joined by `\n`; values containing a comma or quote are quoted.
- `to_list()`: a copy of the row list.

## Compared with tui and menus

[tui](./tui.md) has a print-only `table(headers, rows)` and [menus](./menus.md) has a small `Table(headers)` class that only renders; neither can filter, sort or join. [cli](./cli.md) has `CLITable` for a styled, centred display. Use this module when you need to manipulate data; use the others to show a fixed grid. Note that `menus` also defines a class named `Table`, so do not fetch both in one program.

## Example

```mdhavers
fetch "stdlib/table"

ken t = Table(["name", "age"])
t.add_row(["Ann", 30])
t.add_row({"name": "Bob", "age": 25})
t.add_rows([["Cy", 41], ["Di", 25]])

t.sort_by("age").display()
blether t.sum("age")
blether t.filter(|r| r["age"] > 26).to_csv()
```
