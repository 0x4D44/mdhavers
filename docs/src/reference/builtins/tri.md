# tri (3D scene module)

`tri` is a small 3D scene-graph module with a three.js shaped API and Scots names. You build a scene from objects, give it a camera and a renderer, and the backend draws it.

How much `tri` does depends on the backend. In the interpreter it only records the scene (see [Backends](#backends)).

## Importing

`tri` is built in. Import it with an alias; the alias is required.

```mdhavers
fetch "tri" tae tri
blether tri.DEG_TO_RAD
```

The path may be written `"tri"` or `"tri.braw"` (the JavaScript compiler also accepts `"tri.js"`). The interpreter and the JavaScript and native compilers all reject an import with no alias: `tri import requires an alias (fetch "tri" tae name)`. The WASM compiler fails with `WASM import requires an alias (fetch "tri" tae name)`.

## Constants

| Name | Value |
|---|---|
| `tri.DEG_TO_RAD` | pi / 180, multiply degrees by it to get radians |
| `tri.RAD_TO_DEG` | 180 / pi |

Asking for any name that is not a constant or a constructor fails with an undefined-name error. Assigning to the module (`tri.thing = 1`) fails with `Cannae set 'thing' on tri module`.

## Constructors

Every constructor is a property of the module and also callable as a method, for example `tri.Sicht()`. They return an object. The tables list the properties each one starts with. Missing arguments take the default shown, and extra arguments are ignored.

### Objects with a transform

These all start with `type` (the constructor name), `position`, `rotation`, `scale`, `children` (an empty list) and `parent` (`naething`).

- `position` and `scale` hold `x`, `y`, `z` floats. `position` starts at 0, 0, 0 and `scale` at 1, 1, 1.
- `rotation` holds `x`, `y`, `z` in radians, starting at 0, 0, 0.

| Constructor | Arguments | Extra properties |
|---|---|---|
| `tri.Sicht()` | none | the scene root (three.js "Scene") |
| `tri.Thing3D()` | none | a plain object |
| `tri.Clump()` | none | a group of objects |
| `tri.Mesch(geometry, material)` | geometry, material (both default `naething`) | `geometry`, `material` |
| `tri.Kamera()` | none | a bare camera |
| `tri.PerspectivKamera(fov, aspect, near, far)` | defaults 50, 1, 0.1, 2000 | `fov`, `aspect`, `near`, `far` |
| `tri.OrthograffikKamera(left, right, top, bottom, near, far)` | defaults -1, 1, 1, -1, 0.1, 2000 | the six named properties |
| `tri.Licht(colour, intensity)` | defaults `naething`, 1 | `color`, `intensity` |
| `tri.AmbiantLicht(colour, intensity)` | defaults `naething`, 1 | `color`, `intensity` |
| `tri.DireksionalLicht(colour, intensity)` | defaults `naething`, 1 | `color`, `intensity` |
| `tri.PyntLicht(colour, intensity, distance, decay)` | defaults `naething`, 1, 0, 2 | `color`, `intensity`, `distance`, `decay` |

The light colour argument is stored in the property `color`.

### Geometries, materials and others

These have only `type` and the properties shown. They have no transform.

| Constructor | Arguments | Properties |
|---|---|---|
| `tri.Geometrie()` | none | an empty geometry |
| `tri.BoxGeometrie(width, height, depth)` | defaults 1, 1, 1 | `width`, `height`, `depth` |
| `tri.SpherGeometrie(radius, widthSegments, heightSegments)` | defaults 1, 8, 6 | `radius`, `widthSegments`, `heightSegments` |
| `tri.Maiterial(opts)` | an options dict, default `naething` | `opts` |
| `tri.MeshBasicMaiterial(opts)` | as above | `opts` |
| `tri.MeshStandardMaiterial(opts)` | as above | `opts` |
| `tri.Textur()` | none | a texture placeholder |
| `tri.Renderar(opts)` | an options dict such as `{"width": 800, "height": 600}` | `opts` |
| `tri.Colour(value)` | a colour value, default `naething` | `value` |

## Properties and methods

You can read and write any property by name, and setting a name that does not exist creates it. Reading a name that was never set fails with an undefined-name error.

These methods work on any object returned by a constructor. English alternatives are in the second column.

| Method | Also | What it does in the interpreter |
|---|---|---|
| `adde(item, ...)` | `add` | Appends each argument to `children`. |
| `remuiv(item, ...)` | `remove` | Removes the matching items from `children`. |
| `cloan()` | `clone` | Returns a copy of the object. The copy shares its `position`, `rotation`, `scale` and `children` objects with the original. |
| `dyspos()` | `dispose` | Does nothing. |
| `luik_at(target)` | `lookAt` | Stores the first argument as `lookAtTarget`. It does not rotate anything. |
| `set_sise(width, height)` | `setSize` | Stores `width` and `height`. |
| `set_pixel_ratio(ratio)` | `setPixelRatio` | Stores `pixelRatio`. |
| `render(scene, camera)` | | Stores `scene` and `camera`. |
| `loop(callback)` | | Stores the callback as `loopFn`. It is not called. |

Any other method name does nothing and returns `naething`.

## Example

This runs in the interpreter. It builds a scene and reads values back.

```mdhavers
fetch "tri" tae tri

ken sicht = tri.Sicht()
ken kamera = tri.PerspectivKamera(60, 16 / 9, 0.1, 100)
ken cube = tri.Mesch(tri.BoxGeometrie(2, 1, 1), tri.MeshStandardMaiterial({"color": "#ff8844"}))

cube.position.x = 1.5
cube.rotation.y = 90 * tri.DEG_TO_RAD
sicht.adde(cube)
sicht.adde(tri.AmbiantLicht("#404040", 0.7))

blether kamera.fov
blether cube.geometry.width
blether len(sicht.children)

ken renderar = tri.Renderar({"width": 800, "height": 600})
renderar.render(sicht, kamera)
blether renderar.scene.type
```

It prints `60`, `2`, `2` and `Sicht`.

A render loop looks like this. In the JavaScript backend `renderar.loop(callback)` hands the callback to three.js `setAnimationLoop`, so it runs every frame. Its argument is three.js's frame timestamp divided by 1000: the time in seconds since the page started, not the time since the last frame (`runtime/js/core_runtime.js:3143-3145`). More scenes are in `examples/features/graphics3d_basics.braw` and `examples/showcases/tri_showcase/`.

```mdhavers
fetch "tri" tae tri

ken sicht = tri.Sicht()
ken kamera = tri.PerspectivKamera(60, 16 / 9, 0.1, 100)
ken renderar = tri.Renderar({"width": 800, "height": 600})
ken cube = tri.Mesch(tri.BoxGeometrie(1, 1, 1), tri.MeshStandardMaiterial({"color": "#ff8844"}))
sicht.adde(cube)
kamera.position.z = 4

ken angle = 0.0
renderar.loop(|dt| {
    angle = angle + (0.8 * dt)
    cube.rotation.y = angle
    renderar.render(sicht, kamera)
})
```

In the interpreter this second program stores the callback and finishes without calling it.

## Backends

| Backend | What `tri` is |
|---|---|
| Interpreter | A recording object, as in the tables above. Nothing is drawn. |
| JavaScript | Each constructor builds the matching three.js object (`Sicht` is a `Scene`, `Renderar` a `WebGLRenderer`, `PerspectivKamera` a `PerspectiveCamera`, `Maiterial` a `Material`, and so on). three.js must be available as `globalThis.THREE` or as the `three` package, or the output throws `three.js (THREE) is required for tri`. The Scots method names map to the three.js ones. |
| LLVM/native | `fetch "tri" tae name` binds a native `tri` module object. An alias is required. The examples ask for `--features graphics3d` when building. |
| WAT/WASM | Importing `tri` sets up a module object. The WASM runner provides only `DEG_TO_RAD` and `RAD_TO_DEG` on it (`src/wasm_runner.rs:3665-3678`); constructors are untested. |
| Any backend using `stdlib/tri.braw` | The stub in `stdlib/tri.braw` has the same names and defaults, as dicts with attached methods. |
