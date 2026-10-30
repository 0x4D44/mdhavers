# math

Vectors, matrices, geometric shapes, curves and collision helpers, written as mdhavers classes. It uses the built-in `PI`, `sqrt`, `sin`, `cos`, `atan2`, `acos`, `abs` and `clamp`, so it runs on every backend.

Import: `fetch "stdlib/math"`

Importing prints `Math module loaded! Vectors, matrices, an' geometry at yer service!`. Nothing in this module hurls an error; invalid input (for example a non-vector argument) fails with the normal runtime error. Angles are in radians. All vector and matrix methods return new objects and leave the original untouched, except `Mat3.set` and `Mat3.set_values`. Helpers starting with `_` are internal and not documented.

Integer division truncates in mdhavers, so `Rectangle.center`, `Triangle.centroid`, `polygon_centroid` and `lines_intersect` lose fractions when given integer coordinates, e.g. `Rectangle(0, 0, 1, 1).center()` is `Vec2(0, 0)` but `Rectangle(0.0, 0.0, 1.0, 1.0).center()` is `Vec2(0.5, 0.5)`. Pass floats when you need exact fractional results.

## Functions

### vec2_from_angle(angle, length = 1.0)
`Vec2` pointing along `angle` with the given length.

### vec2_from_list(lst)
`Vec2(lst[0], lst[1])`.

### vec3_from_list(lst)
`Vec3(lst[0], lst[1], lst[2])`.

### mat2_identity()
The 2x2 identity `Mat2`.

### mat2_rotation(angle)
`Mat2` rotating counter-clockwise by `angle`.

### mat2_scale(sx, sy)
Diagonal `Mat2(sx, 0, 0, sy)`.

### mat3_identity()
Identity `Mat3`.

### mat3_translation(tx, ty)
`Mat3` that translates points by `(tx, ty)`.

### mat3_rotation(angle)
`Mat3` rotating by `angle`.

### mat3_scale(sx, sy)
`Mat3` scaling by `(sx, sy)`.

### distance_2d(x1, y1, x2, y2)
Distance between two coordinate pairs (plain numbers, not `Vec2`).

### angle_between_points(x1, y1, x2, y2)
`atan2(y2 - y1, x2 - x1)`.

### are_collinear(p1, p2, p3)
`aye` if the three `Vec2` points lie on one line (cross product under 0.0001).

### angle_between_vectors(v1, v2)
Angle in radians between two vectors; 0 if either has zero length.

### polygon_area(vertices)
Signed area of a list of `Vec2` (positive when counter-clockwise). Returns 0 for fewer than 3 vertices.

### polygon_perimeter(vertices)
Sum of edge lengths, closing back to the start. Returns 0 for fewer than 2 vertices.

### polygon_centroid(vertices)
Average of the vertices as a `Vec2` (`Vec2(0, 0)` when empty). This is the vertex mean, not the area centroid.

### is_convex(vertices)
`aye` if every turn has the same direction. Fewer than 3 vertices gives `nae`.

### bezier_point(p0, p1, p2, p3, t)
Point on a cubic Bezier curve at `t` in 0..1 (all points `Vec2`).

### bezier_quadratic(p0, p1, p2, t)
Point on a quadratic Bezier curve.

### catmull_rom(p0, p1, p2, p3, t)
Catmull-Rom spline point between `p1` and `p2`.

### smoothstep(edge0, edge1, x)
Smooth 0..1 ramp (`3t^2 - 2t^3`) of `x` between the edges.

### smootherstep(edge0, edge1, x)
Perlin's smoother 0..1 ramp.

### aabb_from_points(points)
Bounding `Rectangle` of a list of `Vec2`; `naething` for an empty list.

### aabb_overlap(a, b)
`a.overlaps(b)` for two `Rectangle`s.

### point_in_circle(point, circle)
`circle.contains_point(point)`.

### point_in_rect(point, rect)
`rect.contains_point(point)`.

### circles_collide(c1, c2)
`c1.overlaps(c2)` for two `Circle`s.

### lines_intersect(line1, line2)
Intersection `Vec2` of two `LineSegment`s, or `naething` if they are parallel or do not cross.

## Classes

### Vec2
`Vec2(x = 0.0, y = 0.0)`; fields `x`, `y`. Methods:

- `add(other)`, `sub(other)`: new `Vec2`.
- `scale(s)`: multiply by a number.
- `dot(other)`: scalar dot product.
- `cross(other)`: scalar 2D cross product.
- `length()`, `length_squared()`.
- `normalize()`: unit vector, or `Vec2(0, 0)` for a zero vector.
- `distance(other)`: distance to another vector.
- `angle()`: `atan2(y, x)`.
- `rotate(angle)`: rotated copy.
- `perp()`: 90 degrees counter-clockwise.
- `neg()`: negated.
- `lerp(other, t)`: interpolation.
- `reflect(normal)`: reflect off a unit normal.
- `project(other)`: projection onto `other` (`Vec2(0, 0)` if `other` is zero).
- `equals(other)`: exact component equality.
- `tae_string()`: `Vec2(x, y)`.
- `tae_list()`: `[x, y]`.

### Vec3
`Vec3(x = 0.0, y = 0.0, z = 0.0)`; fields `x`, `y`, `z`. Methods: `add`, `sub`, `scale`, `dot`, `cross` (returns a `Vec3`), `length`, `length_squared`, `normalize` (zero vector gives `Vec3(0, 0, 0)`), `distance`, `neg`, `lerp`, `reflect`, `equals`, `tae_string`, `tae_list`. Each works like its `Vec2` namesake.

### Mat2
`Mat2(a = 1.0, b = 0.0, c = 0.0, d = 1.0)` laid out as rows `[a b]` and `[c d]`; fields `a`, `b`, `c`, `d`. Methods:

- `mul(other)`: matrix product, a new `Mat2`.
- `transform(v)`: apply to a `Vec2`.
- `scale(s)`: scalar multiply.
- `det()`: determinant.
- `inverse()`: inverse, or `naething` when the determinant is 0.
- `transpose()`.
- `tae_string()`: `Mat2([a, b], [c, d])`.

### Mat3
`Mat3()` starts as the identity; field `m` is a row-major list of 9 numbers. Used for 2D transforms with translation. Methods:

- `set_values(m00, m01, m02, m10, m11, m12, m20, m21, m22)`: replace all nine values, returns the matrix.
- `get(row, col)`, `set(row, col, val)`: element access (`set` mutates).
- `mul(other)`: matrix product, a new `Mat3`.
- `transform_point(v)`: apply to a `Vec2` including translation.
- `transform_vector(v)`: apply to a `Vec2` ignoring translation.
- `transpose()`.
- `tae_string()`.

### Rectangle
`Rectangle(x = 0, y = 0, width = 0, height = 0)`. Methods: `area()`, `perimeter()`, `center()` (a `Vec2`), `contains_point(p)` (edges inclusive), `overlaps(other)` (touching edges do not overlap), `corners()` (four `Vec2`: top-left, top-right, bottom-right, bottom-left), `tae_string()`.

### Circle
`Circle(x = 0, y = 0, radius = 1)`. Methods: `area()`, `circumference()`, `center()`, `diameter()`, `contains_point(p)`, `overlaps(other)`, `point_at_angle(angle)`, `tae_string()`.

### LineSegment
`LineSegment(x1 = 0, y1 = 0, x2 = 0, y2 = 0)`; fields `start` and `end` (`Vec2`). Methods: `length()`, `midpoint()`, `direction()` (unit `Vec2`), `point_at(t)` (0 = start, 1 = end), `distance_to_point(p)`, `tae_string()`.

### Triangle
`Triangle(p1, p2, p3)` of `Vec2` points. Methods: `area()`, `perimeter()`, `centroid()`, `contains_point(p)` (barycentric test), `vertices()` (list of the three points), `tae_string()`. A degenerate triangle makes `contains_point` divide by zero.

## Constants

`SQRT2`, `SQRT3`, `INV_SQRT2` (1 / sqrt 2), `DEG_TO_RAD` (PI / 180) and `RAD_TO_DEG` (180 / PI).

## Example

```mdhavers
fetch "stdlib/math"

ken a = Vec2(3, 4)
blether a.length()
blether a.add(Vec2(1, 1)).tae_string()
ken tri = Triangle(Vec2(0, 0), Vec2(4, 0), Vec2(0, 3))
blether tri.area()
blether Circle(0, 0, 2).contains_point(Vec2(1, 1))
blether Mat2(1, 2, 3, 4).det()
```

## Compared with maths

[math](./math.md) provides the geometry classes and linear algebra; [maths](./maths.md) provides numeric functions (gcd, primes, statistics, simple circle and triangle formulas). Prefer `math` for anything positional (vectors, transforms, collisions, curves) and `maths` for scalar number work. The two can be imported together. For general grid-shaped data (lists of lists) rather than 2x2 and 3x3 transforms, see [matrix](./matrix.md).
