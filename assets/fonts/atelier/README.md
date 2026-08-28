# Font Raster Atelier Faces

These are unmodified upstream font binaries embedded only in the
`font_raster_atelier` example. The production Poolrooms stack remains CMU
Typewriter Text with the Noto math and symbol fallbacks in the parent font
directory.

The candidate corpus is pinned so a later upstream release cannot silently
alter a raster judgment:

| Family | Upstream revision | Files | License |
| --- | --- | --- | --- |
| CMU Typewriter Text Bold | CM Unicode 0.7.0 | `cmu-typewriter/cmuntb.otf` | SIL OFL 1.1; see `../cmu-typewriter/OFL.txt` |
| iA Writer Mono, Duo, Quattro | `f32c04c3058a75d7ce28919ce70fe8800817491b` | `ia-writer/*.ttf` | `ia-writer/LICENSE.md` |
| IBM Plex Mono and Sans | `bf260093582f04622aacc1e9f9ca604d7ccd0c42` | `ibm-plex/*.ttf` | `ibm-plex/LICENSE.txt` |
| JetBrains Mono | `19371302b95d218af43299bce79ddbddd0bc364d` | `jetbrains-mono/*.ttf` | `jetbrains-mono/OFL.txt` |
| Courier Prime | `7fd585a2dd4c1612c79b3308e300923d1c13df93` | `courier-prime/*.ttf` | `courier-prime/OFL.txt` |

Sources: [CM Unicode on CTAN](https://ctan.org/pkg/cm-unicode),
[iA Fonts](https://github.com/iaolo/iA-Fonts),
[IBM Plex](https://github.com/IBM/plex),
[JetBrains Mono](https://github.com/JetBrains/JetBrainsMono), and
[Courier Prime](https://github.com/quoteunquoteapps/CourierPrime).
