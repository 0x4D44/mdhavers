#!/usr/bin/env python3
"""Report built-in functions registered in the Rust sources that no docs page documents.

A built-in counts as documented when its name appears in prose (outside fenced
code blocks) of a docs/src page, either as `name(` / `name` in inline code or as
a heading.  Pages under docs/src/stdlib/ and reference/stdlib.md are ignored.
Exit status is 1 when anything is undocumented.  Standard library only.
"""
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "src"
DOCS = ROOT / "docs" / "src"
TUPLE_FILES = {"src/graphics.rs", "src/audio/mod.rs"}
IDENT = r"[A-Za-z_][A-Za-z_0-9]*"


def non_test_text(path):
    text = path.read_text(encoding="utf-8")
    m = re.search(r"^#\[cfg\(test\)\]\s*\n(?:#\[[^\n]*\]\s*\n)*mod ", text, re.M)
    return text[: m.start()] if m else text


def registered():
    """Return {name: set(relative source files)}."""
    found = defaultdict(set)
    pats = [
        re.compile(r'NativeFunction::new\(\s*"(%s)"' % IDENT),
        re.compile(r'define_native\(\s*(?:globals|g)\s*,\s*"(%s)"' % IDENT),
        re.compile(r'define_stub\(\s*globals\s*,\s*"(%s)"' % IDENT),
        re.compile(r'^\s*\(\s*"(%s)"\s*,\s*\d+\s*\)\s*,' % IDENT, re.M),
        re.compile(r'\.define\(\s*"(%s)"\.to_string\(\)' % IDENT),
    ]
    for path in sorted(SRC.rglob("*.rs")):
        rel = str(path.relative_to(ROOT))
        text = non_test_text(path)
        for pat in pats:
            if pat.flags & re.M and rel not in TUPLE_FILES:
                continue
            for m in pat.finditer(text):
                found[m.group(1)].add(rel)
        if path.name == "interpreter.rs":
            # higher-order builtins dispatched by name in call_builtin_hof
            start = text.find("fn call_builtin_hof")
            end = text.find("\n    fn ", start + 10)
            for m in re.finditer(r'^\s{12}"(%s)"((?:\s*\|\s*"%s")*)\s*=>' % (IDENT, IDENT), text[start:end], re.M):
                found[m.group(1)].add(rel)
                for alt in re.findall(r'"(%s)"' % IDENT, m.group(2)):
                    found[alt].add(rel)
        if path.name == "tri.rs":
            for m in re.finditer(r'"(%s)"\s*=>\s*Some\("' % IDENT, text):
                found["tri." + m.group(1)].add(rel)
            for m in re.finditer(r'constants\.insert\("(%s)"' % IDENT, text):
                found["tri." + m.group(1)].add(rel)
    # test-only placeholder names registered in value.rs are not built-ins
    for junk in ("native",):
        found.pop(junk, None)
    return found


def documented_text():
    chunks = []
    for md in sorted(DOCS.rglob("*.md")):
        rel = md.relative_to(DOCS).as_posix()
        if rel.startswith("stdlib/") or rel == "reference/stdlib.md":
            continue
        text = md.read_text(encoding="utf-8")
        text = re.sub(r"^```.*?^```", "", text, flags=re.S | re.M)
        chunks.append(text)
    return "\n".join(chunks)


def is_documented(name, prose):
    if name.startswith("tri."):
        n = re.escape(name[4:])
        return re.search(r"`(?:tri\.)?%s\b[^`]*`" % n, prose) is not None
    n = re.escape(name)
    return re.search(r"`[^`\n]*(?<![A-Za-z_0-9])%s(?![A-Za-z_0-9])[^`\n]*`" % n, prose) is not None


def main():
    reg = registered()
    prose = documented_text()
    missing = defaultdict(list)
    for name in sorted(reg):
        if not is_documented(name, prose):
            for f in sorted(reg[name]):
                missing[f].append(name)
    total = len(reg)
    bad = sum(len({n for n in v}) for v in missing.values())
    for f in sorted(missing):
        print("%s (%d)" % (f, len(missing[f])))
        for n in missing[f]:
            print("    " + n)
    undocumented = {n for v in missing.values() for n in v}
    print("built-ins found: %d  documented: %d  undocumented: %d"
          % (total, total - len(undocumented), len(undocumented)))
    return 1 if undocumented else 0


if __name__ == "__main__":
    sys.exit(main())
