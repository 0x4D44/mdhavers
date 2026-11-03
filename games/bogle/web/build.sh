#!/bin/bash
# Build Bogle Castle for the browser: index.html plus bogle.js/.wasm/.data from
# games/bogle/shell/web.braw, which hands its frames to the page (screen_loop), so no
# JSPI is needed. Serve the output directory over HTTP, e.g.
#   python3 -m http.server -d target/bogle-web 8000
#
# Usage: games/bogle/web/build.sh [--jspi] [--out DIR] [mdhavers build args...]
#   --jspi     also build bogle_jspi.* from the blocking shell (native.braw, whose
#              snooze suspends through JSPI); the page loads it with ?jspi
#   --out DIR  output directory (default target/bogle-web in the repository)
# The build runs inside the output directory, so give any path in the extra
# arguments as an absolute one.
# Needs emcc (Emscripten) on PATH and a built compiler: $MDHAVERS, else
# target/release/mdhavers (cargo build --release).
set -euo pipefail

web=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$web/../../.." && pwd)
out="$repo/target/bogle-web"
jspi=0
while [ $# -gt 0 ]; do
  case "$1" in
    --jspi) jspi=1; shift ;;
    --out) out=$2; shift 2 ;;
    *) break ;;
  esac
done
mdhavers=${MDHAVERS:-$repo/target/release/mdhavers}
if [ ! -x "$mdhavers" ]; then
  echo "build.sh: nae compiler at $mdhavers (cargo build --release, or set MDHAVERS)" >&2
  exit 1
fi
mkdir -p "$out"
out=$(cd "$out" && pwd)

# The game reads its data relative to the repository root, so preload it at those
# paths. Build from inside the output directory with a bare --output name: emcc writes
# the .data path it was given into bogle.js, and an absolute one would carry this
# machine's folders onto the page.
cd "$out"
export MDHAVERS_HOME=${MDHAVERS_HOME:-$repo}
preload=(
  --emcc-arg=--preload-file --emcc-arg="$repo/games/bogle/data@games/bogle/data"
  --emcc-arg=--preload-file --emcc-arg="$repo/games/bogle/art/palette.txt@games/bogle/art/palette.txt"
)
"$mdhavers" build --target wasm32-emscripten "$repo/games/bogle/shell/web.braw" \
  --output bogle.js "${preload[@]}" "$@"
if [ "$jspi" = 1 ]; then
  "$mdhavers" build --target wasm32-emscripten "$repo/games/bogle/shell/native.braw" \
    --output bogle_jspi.js "${preload[@]}" --emcc-arg=-sJSPI "$@"
fi
cp "$web/index.html" "$out/index.html"
echo "Bogle's page is in $out: serve it wi' python3 -m http.server -d \"$out\" 8000"
