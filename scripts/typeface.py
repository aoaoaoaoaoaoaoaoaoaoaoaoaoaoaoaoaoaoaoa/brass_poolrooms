#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.14"
# dependencies = ["fonttools==4.64.0", "pillow==12.1.0"]
# ///
"""Build and prove the owned glyph map without rewriting the base outlines."""

from __future__ import annotations

import argparse
import os
import subprocess
import tempfile
from pathlib import Path

from fontTools.pens.t2CharStringPen import T2CharStringPen
from fontTools.ttLib import TTFont
from fontTools.ttLib.tables._c_m_a_p import CmapSubtable
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets/fonts"
BASE = ASSETS / "cmu-typewriter/cmunbtl.otf"
MASTER = ASSETS / "poolrooms/glyph-map.sfd"
OUTPUT = ASSETS / "poolrooms/PoolroomsTypewriter-Light.otf"
FAMILY = "Poolrooms Typewriter"
FULL_NAME = f"{FAMILY} Light"
POSTSCRIPT = "PoolroomsTypewriter-Light"


def marks() -> str:
    return "".join(
        chr(int(fields[2]))
        for line in MASTER.read_text().splitlines()
        if line.startswith("Encoding: ")
        and len(fields := line.split()) == 4
        and int(fields[2]) >= 0
    )


def mapping(font: TTFont) -> dict[int, str]:
    cmap = font.getBestCmap()
    if cmap is None:
        raise ValueError("font lacks a Unicode cmap")
    return cmap


def forge() -> None:
    with tempfile.TemporaryDirectory(prefix="poolrooms-typeface-") as temporary:
        extension = Path(temporary) / "marks.otf"
        environment = {
            key: value
            for key, value in os.environ.items()
            if key not in {"DISPLAY", "WAYLAND_DISPLAY"}
        }
        subprocess.run(
            [
                "fontforge",
                "-lang=py",
                "-c",
                "import fontforge; f=fontforge.open(argv[1]); "
                "errors={g.glyphname: g.validate() "
                "for g in f.glyphs() if g.validate()}; "
                "assert not errors, errors; "
                "f.generate(argv[2]); f.close()",
                str(MASTER),
                str(extension),
            ],
            env=environment,
            check=True,
        )
        with TTFont(BASE, recalcTimestamp=False) as font, TTFont(extension) as extra:
            cff = font["CFF "].cff
            top = cff.topDictIndex[0]
            strings = top.CharStrings
            outlines = extra.getGlyphSet()
            cmap = dict(mapping(font))
            for scalar, source in sorted(mapping(extra).items()):
                name = f"u{scalar:05X}.poolrooms"
                width = extra["hmtx"].metrics[source][0]
                pen = T2CharStringPen(width - top.Private.nominalWidthX, dict(outlines))
                outlines[source].draw(pen)
                strings.charStringsIndex.append(
                    pen.getCharString(
                        private=top.Private,
                        globalSubrs=[
                            cff.GlobalSubrs[i] for i in range(len(cff.GlobalSubrs))
                        ],
                    )
                )
                strings.charStrings[name] = len(strings.charStringsIndex) - 1
                top.charset.append(name)
                font["hmtx"].metrics[name] = (width, extra["hmtx"].metrics[source][1])
                cmap[scalar] = name
            font.setGlyphOrder(top.charset)
            font["maxp"].numGlyphs = len(top.charset)
            for table in font["cmap"].tables:
                if table.isUnicode():
                    table.cmap = {
                        scalar: name
                        for scalar, name in cmap.items()
                        if table.format == 12 or scalar <= 0xFFFF
                    }
            if not any(table.format == 12 for table in font["cmap"].tables):
                full = CmapSubtable.newSubtable(12)
                full.platformID, full.platEncID, full.language = 3, 10, 0
                full.cmap = cmap
                font["cmap"].tables.append(full)
            replacements = {
                1: FAMILY,
                2: "Light",
                3: POSTSCRIPT,
                4: FULL_NAME,
                6: POSTSCRIPT,
                16: FAMILY,
                17: "Light",
            }
            for entry in font["name"].names:
                if entry.nameID in replacements:
                    entry.string = replacements[entry.nameID].encode(
                        entry.getEncoding()
                    )
                elif entry.nameID == 0:
                    entry.string = (
                        entry.toUnicode()
                        + "\nOriginal marks copyright 2026 Poolrooms contributors."
                    ).encode(entry.getEncoding())
            cff.fontNames[0] = POSTSCRIPT
            top.FamilyName, top.FullName = FAMILY, FULL_NAME
            font["OS/2"].recalcUnicodeRanges(font)
            font.save(OUTPUT)
    check()


def check() -> None:
    with TTFont(BASE) as base, TTFont(OUTPUT) as font:
        base_cmap, cmap = mapping(base), mapping(font)
        admitted = {ord(char) for char in marks()}
        if missing := admitted - cmap.keys():
            raise ValueError(f"glyph map missing from generated font: {missing}")
        if missing := base_cmap.keys() - cmap.keys():
            raise ValueError(f"base cmap lost: {missing}")
        original = base["CFF "].cff.topDictIndex[0].CharStrings
        generated = font["CFF "].cff.topDictIndex[0].CharStrings
        for scalar, name in base_cmap.items():
            if scalar in admitted:
                continue
            if (
                cmap[scalar] != name
                or base["hmtx"].metrics[name] != font["hmtx"].metrics[name]
            ):
                raise ValueError(f"base mapping or metric changed: U+{scalar:04X}")
            if original[name].bytecode != generated[name].bytecode:
                raise ValueError(f"base charstring changed: U+{scalar:04X}")
        if len(font.getGlyphOrder()) != len(set(font.getGlyphOrder())):
            raise ValueError("duplicate glyph names")
    print(
        f"Typeface: {len(admitted)} designed marks; "
        "base mappings, metrics and charstrings preserved"
    )


def proof(path: Path) -> None:
    characters = list(marks())
    width, cell, margin = 1440, 90, 30
    columns = (width - margin * 2) // cell
    rows = (len(characters) + columns - 1) // columns
    image = Image.new("RGB", (width, 160 + rows * 275), "#eee9dd")
    draw = ImageDraw.Draw(image)
    ink, muted = "#201b15", "#9d9280"
    caption = ImageFont.truetype(str(OUTPUT), 18)
    draw.text((margin, 20), "Poolrooms Typewriter Light", font=caption, fill=ink)
    draw.text(
        (margin, 60),
        "H O n o 0  + /    O ∅ 0    Copy / Paste / Settings",
        font=ImageFont.truetype(str(OUTPUT), 24),
        fill=ink,
    )
    for index, char in enumerate(characters):
        x = margin + index % columns * cell
        y = 120 + index // columns * 275
        draw.text((x, y), f"{ord(char):04X}", font=caption, fill=muted)
        for offset, size in [(35, 14), (75, 20), (125, 30), (185, 50)]:
            draw.text(
                (x + 20, y + offset),
                char,
                font=ImageFont.truetype(str(OUTPUT), size),
                fill=ink,
            )
    image.save(path)
    print(path)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("forge", "check", "proof"))
    parser.add_argument("output", type=Path, nargs="?")
    args = parser.parse_args()
    match args.action:
        case "forge":
            forge()
        case "check":
            check()
        case "proof":
            if args.output is None:
                parser.error("proof requires an output PNG path")
            proof(args.output)


if __name__ == "__main__":
    main()
