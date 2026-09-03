# Font Raster Atelier Faces

These are unmodified upstream font binaries embedded in the
`font_raster_atelier` example. The selected CMU Typewriter Text Light cut has
graduated to `../cmu-typewriter/` for shared production and atelier use; other
files here remain candidates. Production obtains mathematical and symbol
coverage from the vendored Noto faces in the parent font directory.

The candidate corpus is pinned so a later upstream release cannot silently
alter a raster judgment:

| Family | Upstream revision | Files | License |
| --- | --- | --- | --- |
| CMU Typewriter Text | CM Unicode 0.7.0 | `cmu-typewriter/cmuntt.ttf`, `cmu-typewriter/cmuntb.otf`, `../cmu-typewriter/cmunbtl.otf` | SIL OFL 1.1; see `../cmu-typewriter/OFL.txt` |
| Computer Modern Graded Typewriter | CTAN 1.0.0 | `cm-graded/*.otf` | `cm-graded/OFL.txt` |
| Latin Modern Mono | CTAN 2.005 | `latin-modern/*.otf` | `latin-modern/GUST-FONT-LICENSE.txt` |
| New Computer Modern Mono | CTAN 8.1.1 | `new-computer-modern/*.otf` | `new-computer-modern/LICENSE.txt` |
| Courier Prime | `7fd585a2dd4c1612c79b3308e300923d1c13df93` | `courier-prime/*.ttf` | `courier-prime/OFL.txt` |
| Courier Prime Code | `0fcb44c7bcb7e81079dfa9f0d5ac5a4b3e7bf853` | `courier-prime-code/*.ttf` | `courier-prime-code/LICENSE.md` |
| TeX Gyre Cursor | CTAN 2.004 | `tex-gyre-cursor/*.otf` | `tex-gyre-cursor/GUST-FONT-LICENSE.txt` |
| Nimbus Mono PS | URW Base 35 `20200910` (`c15105598aa7eb256b1ebfcecd3d078801521e73`) | `nimbus-mono/*.otf` | `nimbus-mono/LICENSE`, `nimbus-mono/COPYING` |
| Liberation Mono | Arch `ttf-liberation` 2.1.5-2, upstream 2.1.5 | `liberation-mono/*.ttf` | `liberation-mono/LICENSE` |
| Cousine | Google Fonts `ade3d1533e06b2b1462ffcde8e08b129627ca360` | `cousine/*.ttf` | `cousine/OFL.txt` |

The census intentionally admits only upright normal-text cuts from the two
relevant bloodlines. The Computer Modern side contains both CM Unicode
typewriter weights, all seven metric-preserving CM Graded darkness grades,
every Latin Modern upright optical master, its light and light-condensed cuts,
and New Computer Modern Mono. The Courier side contains the original URW
master, its TeX Gyre extension, the Courier Prime redraw and code cut, and the
two principal Courier New metric-compatible substitutes. Italic, oblique,
small-cap, proportional typewriter, and unrelated coding-sans cuts are not
candidate application faces. Type 1-only CM derivatives cannot enter this
native epaint/skrifa judgment surface.

Sources: [CM Unicode on CTAN](https://ctan.org/pkg/cm-unicode),
[Computer Modern Graded](https://ctan.org/pkg/cmgraded),
[Latin Modern](https://ctan.org/pkg/lm),
[New Computer Modern](https://ctan.org/pkg/newcomputermodern),
[Courier Prime](https://github.com/quoteunquoteapps/CourierPrime),
[Courier Prime Code](https://github.com/quoteunquoteapps/CourierPrimeCode),
[TeX Gyre Cursor](https://ctan.org/pkg/tex-gyre-cursor),
[URW Base 35](https://github.com/ArtifexSoftware/urw-base35-fonts),
[Liberation Fonts](https://github.com/liberationfonts/liberation-fonts), and
[Cousine](https://github.com/google/fonts/tree/main/ofl/cousine).
