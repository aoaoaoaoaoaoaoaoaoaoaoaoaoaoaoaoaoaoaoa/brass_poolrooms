# Poolrooms Typewriter Light

One production face, with no fallback. The text comes from CMU Typewriter Text
Light; its unchanged glyph programs, mappings, metrics, and hinting survive
assembly byte-for-byte. New marks are original FontForge drawings, not outlines
imported from another symbol font. The empty-set mark and subscripts reuse the
base face's own letters and digits.

`glyph-map.sfd` is the editable master for additions. `PoolroomsTypewriter-Light.otf`
is the generated production face. The master uses a 1000-unit em, a 611-unit cap
height, a 525-unit normal advance, and strokes tuned for small engraved controls.
Its repertoire covers the action armory, disclosure triangles, selection dot,
keyboard marks, empty set, mathematical operators, and subscripts. The SFD
owns the exact Unicode mappings. The typeface does not claim CJK coverage;
the font atelier's sample prose stays within the supported scripts.

## Design Loop

Open the master in FontForge, or edit it with FontForge's Python API. Preserve
few intentional contours, open counters, and the base face's narrow proportions.
Validate intersections, winding, extrema, bearings, and mapping before export.
The FontForge application is an authoring dependency, not a runtime dependency.

From the repository root, with FontForge and uv installed:

```sh
uv run scripts/typeface.py forge
uv run scripts/typeface.py check
uv run scripts/typeface.py proof /tmp/poolrooms-typeface.png
```

Inspect the proof at native size and the same glyphs in native Poolrooms
monoglyphs and tapes. Pixel proofs judge small-size legibility; enlarged
outlines alone do not. The builder refuses invalid contours and checks that
unmodified CMU glyphs survive assembly. The `glyph-audit` Rust feature checks
authored string and character literals against the exported cmap, including
escaped literals and macro arguments. Documentation and test bodies are outside
that UI contract. User-entered text is preserved, not validated as source code.

## License

This renamed derivative retains the CMU authors' notices and
[SIL Open Font License 1.1](../cmu-typewriter/OFL.txt). Original Poolrooms marks
are copyright 2026 Poolrooms contributors, under the same license. Reserved
upstream font names are not used for the derivative's family or PostScript name.
