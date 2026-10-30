# matrix

Helpers for matrices and grids stored as a list of rows (a list of lists). It covers creation, arithmetic, row and column work, statistics, rotations and flips, neighbour lookups for games, flood fill, console display and a few special matrices. Works on every backend. For 2x2 and 3x3 transform classes see [math](./math.md).

Import: `fetch "stdlib/matrix"`

Importing prints `Matrix module loaded! Rows an' columns, like a tartan weave!`. Nothing here hurls a custom error: out-of-range indices or ragged rows fail with the normal runtime error. Indices are zero-based, `[row][col]`. Helpers starting with `_` are internal and not documented.

Mutation: `set_element`, `set_row`, `set_col`, `swap_rows`, `swap_cols`, `place_on_grid` and `flood_fill` change the list you pass in and also return it. Every other function returns a new matrix.

## Functions

### matrix(rows, cols, fill_val = 0)
New `rows` by `cols` matrix filled with `fill_val`.

### identity_matrix(size)
Square matrix with 1 on the diagonal and 0 elsewhere.

### matrix_from_rows(rows)
Copy of a list of rows.

### matrix_from_flat(flat_list, rows, cols)
Reshape a flat list row by row; missing values become 0.

### num_rows(m)
Row count.

### num_cols(m)
Column count (0 for an empty matrix).

### dimensions(m)
`[rows, cols]`.

### is_square(m)
`aye` when rows equal columns.

### is_empty(m)
`aye` when there are no rows or no columns.

### get_element(m, row, col)
`m[row][col]`.

### set_element(m, row, col, val)
Sets `m[row][col]` in place and returns `m`.

### matrix_add(a, b)
Element-wise sum (uses the dimensions of `a`).

### matrix_subtract(a, b)
Element-wise difference.

### scalar_multiply(m, scalar)
Multiply every element by a number.

### matrix_multiply(a, b)
Matrix product; `a` columns must equal `b` rows.

### transpose(m)
Rows become columns.

### get_row(m, row_idx)
Copy of one row.

### get_col(m, col_idx)
One column as a list.

### set_row(m, row_idx, new_row)
Overwrites the row in place, returns `m`.

### set_col(m, col_idx, new_col)
Overwrites the column in place, returns `m`.

### swap_rows(m, row1, row2)
Swaps two rows in place, returns `m`.

### swap_cols(m, col1, col2)
Swaps two columns in place, returns `m`.

### matrix_sum(m)
Sum of all elements.

### row_sums(m)
List of row totals.

### col_sums(m)
List of column totals.

### matrix_max(m)
Largest element (uses `m[0][0]`, so an empty matrix fails).

### matrix_min(m)
Smallest element.

### matrix_average(m)
Sum divided by element count (integer division for integer matrices).

### diagonal(m)
Main diagonal as a list.

### trace(m)
Sum of the diagonal.

### matrix_map(m, func)
New matrix with `func` applied to every element.

### matrix_filter(m, predicate)
Flat list of the elements for which `predicate(value)` is truthy.

### flatten(m)
All elements as one flat list, row by row.

### rotate_clockwise(m)
Rotate 90 degrees clockwise.

### rotate_counter_clockwise(m)
Rotate 90 degrees counter-clockwise.

### flip_horizontal(m)
Mirror left to right.

### flip_vertical(m)
Mirror top to bottom.

### submatrix(m, start_row, start_col, end_row, end_col)
Copy of the block from the start (inclusive) to the end (exclusive).

### concat_horizontal(a, b)
Join side by side (same row count).

### concat_vertical(a, b)
Stack `b` below `a`.

### game_grid(rows, cols, empty_char = ".")
A grid of `empty_char`.

### place_on_grid(grid, row, col, item)
Sets a cell in place and returns the grid.

### in_bounds(grid, row, col)
`aye` if the position is inside the grid.

### neighbors_8(grid, row, col)
In-bounds neighbours in eight directions, as a list of dictionaries `{"row": r, "col": c, "val": v}`.

### neighbors_4(grid, row, col)
Same for up, down, left, right (in that order).

### find_all(grid, target)
List of `[row, col]` positions whose value equals `target`.

### count_in_grid(grid, target)
How many cells equal `target`.

### flood_fill(grid, start_row, start_col, new_val)
Replaces the connected (4-way) region of the start cell's value with `new_val`, in place; returns the grid. Does nothing if the start cell already holds `new_val`.

### display_matrix(m, cell_width = 4)
Prints each row as right-aligned cells between `│` bars.

### display_matrix_with_headers(m)
Prints a table with column numbers across the top and row numbers down the side.

### display_grid(grid)
Prints each row with the cell values joined with no gaps.

### display_grid_bordered(grid)
Same, inside a box drawn with `─`, `│`, `┐` and `┘`.

### checkerboard_matrix(size, val1 = 0, val2 = 1)
Square matrix alternating `val1` and `val2`, with `val1` at the top-left.

### spiral_matrix(n)
`n` by `n` matrix holding 1 to n*n in a clockwise spiral from the top-left.

### magic_square(n)
Magic square for odd `n`. For even `n` it prints `Magic square only works fer odd sizes!` and returns an all-zero matrix.

## Example

```mdhavers
fetch "stdlib/matrix"

ken m = matrix_from_flat([1, 2, 3, 4, 5, 6], 2, 3)
blether m
blether transpose(m)
blether matrix_multiply(m, transpose(m))
blether spiral_matrix(3)
display_matrix(m)
```

## Compared with math

[matrix](./matrix.md) works on plain lists of lists of any size and is about data and grids. [math](./math.md) has fixed-size `Mat2` and `Mat3` classes with transforms for `Vec2` points. Use `matrix` for tables, game boards and general linear algebra on lists; use `math` for 2D geometry transforms.
