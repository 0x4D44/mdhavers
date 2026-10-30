# tri

A stub of the `tri` 3D API, with three.js-like semantics and Scots names: scenes, meshes, cameras, geometries, materials, lights and a renderer, all built as plain dicts. It records state (positions, children, sizes) but draws nothing. The source says native backends should override it with a real implementation (see `src/tri.rs`).

Import: `fetch "stdlib/tri"`

Stub for non-native backends: the dict objects below work everywhere, but only a native tri build renders to a window. This module hurls nothing and prints nothing on load. Helpers starting with `_` are internal and not documented.

Scene-graph objects (`Sicht`, `Thing3D`, `Clump`, `Mesch`, cameras, lights) are dicts with `type`, `position`, `rotation`, `scale` (each `{"x","y","z"}`; scale starts at 1), `children` (list), `parent` (`naething`) and these closures, each with an English alias:

- `adde(item)` / `add(item)`: append to `children`.
- `remuiv(item)` / `remove(item)`: remove from `children`.
- `cloan()` / `clone()`: shallow copy of the dict. The copy's closures (`adde`, `luik_at`, ...) are still bound to the original object, so calling them on the copy changes the original, not the copy.
- `dyspos()` / `dispose()`: does nothing.
- `luik_at(target)` / `lookAt(target)`: stores `lookAtTarget`.

## Constants

`DEG_TO_RAD` (pi / 180) and `RAD_TO_DEG` (180 / pi).

## Functions

### Sicht()
Scene object (`type` `"Sicht"`).

### Thing3D()
Generic 3D object.

### Clump()
Group object.

### Mesch(geometrie = naething, maiterial = naething)
Mesh with `geometry` and `material` fields.

### Kamera()
Base camera.

### PerspectivKamera(fov = 50, aspect = 1, near = 0.1, far = 2000)
Perspective camera with those fields.

### OrthograffikKamera(left = -1, right = 1, top = 1, bottom = -1, near = 0.1, far = 2000)
Orthographic camera with those fields.

### Geometrie()
Empty geometry dict.

### BoxGeometrie(w = 1, h = 1, d = 1)
Dict with `width`, `height`, `depth`.

### SpherGeometrie(r = 1, wseg = 8, hseg = 6)
Dict with `radius`, `widthSegments`, `heightSegments`.

### Maiterial(opts = naething)
Material dict with an `opts` field.

### MeshBasicMaiterial(opts = naething)
Basic material dict.

### MeshStandardMaiterial(opts = naething)
Standard material dict.

### Licht(colour = naething, intensity = 1)
Light (scene object) with `color` and `intensity`.

### AmbiantLicht(colour = naething, intensity = 1)
Ambient light.

### DireksionalLicht(colour = naething, intensity = 1)
Directional light.

### PyntLicht(colour = naething, intensity = 1, distance = 0, decay = 2)
Point light; adds `distance` and `decay`.

### Textur()
Texture dict.

### Renderar(opts = naething)
Renderer dict with these closures: `set_sise(w, h)` / `setSize` (stores `width`, `height`), `set_pixel_ratio(r)` / `setPixelRatio` (stores `pixelRatio`), `render(scene, camera)` (stores `scene` and `camera`), `loop(fn)` (stores `loopFn`). Nothing is drawn.

### Colour(value = naething)
Colour dict `{"type": "Colour", "value": value}`.

## Example

```mdhavers
fetch "stdlib/tri"

ken scene = Sicht()
ken cam = PerspectivKamera(75, 2)
ken box = Mesch(BoxGeometrie(1, 2, 3), MeshBasicMaiterial({"color": 255}))
scene.adde(box)
ken r = Renderar()
r.set_sise(800, 600)
r.render(scene, cam)
blether len(scene.children)
blether [r.width, r.height]
```
