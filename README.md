# Brass Poolrooms

Skeuomorphic controls and living water for egui.

Poolrooms supplies embedded typography, machined bronze chrome, mechanically
constrained controls, and a persistent GPU water surface that reacts to UI
motion. Its custom controls are the linkage-driven [`Rail`], tape-transport
[`DateSpool`], spring-latched [`Checkbox`], and momentary square [`Monoglyph`].
The corner-mounted [`CornerClose`] uses that same plunger stock with a fixed,
deeply recessed X. [`DragHandle`] supplies fixed friction pads and rigid or
folding bails for reorder gestures. The map-anchored [`ForgePin`] unifies its
shaft, spherical grip, hit region, and optional inscription. [`NumberInput`]
couples an exact scalar register to a scalloped thumbwheel. All are machined in
one fixed projection, material, and lighting model.

## Scope

Poolrooms is the independently usable low-level physical GUI substrate. It
owns how controls and surfaces are embodied: geometry, material, constrained
motion, intrinsic interaction, and displaced water. Buttons, rollers, sliders,
tiles, frames, and similar physical things belong here.

Logical application assemblies do not. Managers, menu models, storage
interactions, product layouts, and other application-scale state machines
belong to their application kit, such as Eternalist Apps. Eternalist may
compose Poolrooms; Poolrooms never depends on Eternalist. Its public API and
WebGPU gallery remain usable by applications with entirely different visual
composition.

## Component Index

| Mechanism | Contract | Water hook |
| --- | --- | --- |
| [`Rail`] | Bounded linear transport with explicit total and admissible spans, detents, focused Left/Right/Home/End input, and hovered wheel input by default | `Surface::rail` |
| [`DateSpool`] | One-to-three Gregorian tape reels with explicit width and a reel-derived rigid minimum | `Surface::date_spool` |
| [`Checkbox`] | Three-gauge latching plunger with an optional side-selectable etched plaque and a state-transparent fixed-stock guard | `Surface::checkbox` |
| [`Monoglyph`] | Three-gauge momentary square plunger carrying one etched scalar; [`Symbol`] supplies the canonical common-action armory | `Surface::monoglyph` |
| [`CornerClose`] | Three-gauge momentary close plunger centered on a pane corner, with a build-time modelled and self-shadowed X trench | `Surface::corner_close` |
| [`DragHandle`] | Rigid half-width friction pad, rigid square bail, or sprung folding bail on a riveted crosshatched plate | `Surface::drag_handle` |
| [`ForgePin`] | Three-gauge coordinate pin with one coherent shaft, bulb, grip region, and medium/large inscription API | application-defined |
| [`NumberInput`] | Bounded integer or floating register with an explicit quantum, precision, exact-entry override, two wheel planes, and sprung limit refusal | `Surface::number_input` |

These eight types are the complete inventory of authored foundry mechanisms:
their projected geometry, dynamics, material response, and displaced-water
contract live here. The other `chrome` exports are shared typography, frames,
layout, and interaction assemblies rather than custom projected mechanisms.
[`widget_gallery`](examples/widget_gallery.rs) is the living visual contract:
it keeps one legible exemplar of every material variant and interaction without
multiplying equivalent Cartesian combinations.

[`Section`] embodies the recessed disclosure used by higher-level panel
managers. Its active and focused states are physical indications only;
application logic owns panel selection and traversal. [`MnemonicText`] marks
one permanent Alt glyph, while [`Keycap`] renders a noninteractive chord plate.
`chrome::exact_activation` refines button-like egui responses to pointer,
accessibility, or fresh unmodified Enter/Space activation and leaves modified
chords for their exact owner. Disabled controls and controls behind a modal
layer cannot activate. These parts state a common physical interaction language
without introducing commands, menus, or navigation policy.

[`MechanismSize`] gives compatible `Checkbox`, `Monoglyph`, `CornerClose`,
`DragHandle`, and `ForgePin` dies three named gauges. Cased mechanisms use the
20-point `Small`, 24-point `Medium`, and 32-point `Large` values as their
nominal casing and interaction height, not as a transparent layout envelope.
Monoglyphs and bails are square; friction pads are half-width. Checkbox guards
retain one wire gauge while their lattices step from 2×2 through 3×3 to 4×4,
and their full protective envelopes are allocated explicitly. Forge pins have
their own map-anchored gauge geometry; medium and large bulbs admit native
centered text while small remains unlettered. Each admitted cased gauge is
independently projected and illuminated at build time.
`Monoglyph::symbol(Symbol::Add)` and the other armory constructors bind a
common action to one scalar and the selected foundry gauge's typography. A
product-specific mark may still use `Monoglyph::new(char)`; common add, remove,
duplicate, rename, confirm, disclosure, restore, help, and increment/decrement
marks must not be re-authored at application call sites.
[`Coupled::horizontal`] places any two coupling-capable foundry responses at the
canonical six-point gap and runs the standard twin bronze ties behind both
casings. [`Coupled::horizontal_with_gap`] accepts a [`CouplingGap`] when dense
tool strips need shorter, still-physical ties; `CouplingGap::MINIMUM` is two
points and `CouplingGap::TIGHT` is three.

`NumberInput::new(&mut value, min..=max, quantum, precision)` leaves every
scalar policy with the caller. The bound primitive type selects integer or
floating semantics; integer registers require zero decimal places. Scrolling
one ordinary wheel detent advances one quantum, high-resolution motion retains
its magnitude, and double-clicking the register admits exact text entry.

## Try It

```sh
cargo run --example widget_gallery
cargo run --example slider_gallery
cargo run --example date_spool_gallery
cargo run --example checkbox_gallery
cargo run --example corner_close_gallery
cargo run --example drag_handle_gallery
cargo run --example number_input_gallery
```

The combined gallery is also the browser contract:

```sh
scripts/web-gallery serve
```

Open `http://127.0.0.1:4173`. `scripts/web-gallery build` emits the deployable
static directory under Cargo's target directory. The ordinary repository check
builds and binds this exact release-mode Wasm example, so native-only drift
fails the gate. The server also uses that optimized artifact; debug Wasm is not
representative enough to serve.

For sites that cannot justify the full WebGPU renderer, Poolrooms also owns a
versioned lightweight Web Kit:

```sh
scripts/web-kit check       # rebuild and compare every projection
scripts/web-kit package     # assemble the deterministic release archive
```

It contains static floor and chrome witnesses, a finite analytic WebGL2 water
projection, and generic CSS. The floor is rendered directly from the native
WGSL; chrome is rendered through the current Rust API. Deliberate source locks
stop changes to governing Rust or WGSL until the hand-maintained browser
projection has been reviewed. Consumers pin an exact release and copy selected
assets during their build or synchronization step; no public CDN or runtime
package dependency is required. See [web-kit/README.md](web-kit/README.md).

Pass a port as `scripts/web-gallery serve 8080` or through `PORT=8080`. If that
port is occupied, the server binds an available ephemeral port and prints its
actual URL instead of dying.

For a persistent, fixed address, install the repository-aware systemd user
service:

```sh
scripts/gallery-service install        # http://127.0.0.1:4173
scripts/gallery-service install 8080   # choose another fixed port
```

The installer builds once, adds one integration artifact at
`$XDG_CONFIG_HOME/systemd/user/brass-poolrooms-gallery.service` (falling back
to `~/.config/systemd/user`), and enables it for the user session; the bundle
remains rebuildable Cargo target output. The server reads that output in place:
every later `scripts/web-gallery build` or `./check.py check` is visible on the
next browser refresh without restarting the service. Unlike the interactive
server, a fixed-port collision fails loudly instead of silently changing the
URL. Inspect or remove it with `scripts/gallery-service status` and
`scripts/gallery-service uninstall`.

Linux Chromium may require graphics acceleration plus both
`chrome://flags/#enable-unsafe-webgpu` and `chrome://flags/#enable-vulkan`; relaunch
the browser after changing either flag. The page checks for an actual adapter
before starting and refuses CPU renderers such as `SwiftShader`: the living water
is deliberately a hardware WebGPU workload.

## Use It

```toml
[dependencies]
brass_poolrooms = "0.13.1"
```

Import egui through the crate to keep its public geometry types aligned with
the renderer, then install the chrome once:

```rust
use brass_poolrooms::{chrome, egui};

let ctx = egui::Context::default();
chrome::install(&ctx);
```

For chrome without GPU water:

```toml
brass_poolrooms = { version = "0.13.1", default-features = false }
```

## Water

Water is a post-process over the already-rasterized interface. It therefore
requires a direct egui-wgpu render graph; an eframe paint callback is too late.

1. Record geometry and interaction against a `Surface` during the UI pass.
2. Render egui into `Engine::scene_view()` while the surface is live.
3. Call `Engine::compose()` into the swapchain before submitting.
4. After submission, call `Engine::after_submit()` and honor its repaint request.

Water repaint requests are finite leases, not a frame clock. A newly hovered,
focused, moved, or released mechanism wakes its quiver; identical tension in a
later frame does not renew that wake. Application hosts must likewise stop
rendering concealed windows and suppress frame-originated continuation while
unfocused. Domain progress must be driven by workers, events, or deadlines
rather than by the water simulator.

[`examples/support/mod.rs`](examples/support/mod.rs) is a complete minimal host,
including input, resize, surface recovery, and repaint scheduling. `egui_wgpu`
is re-exported so consumers use the exact wgpu type universe expected by the
engine.

The default `water` feature contains the simulator and compositor.
`instrumentation` adds semantic chrome anchors for deterministic UI driving.
Poolrooms deliberately selects no native wgpu backend for a consuming
application. The application host owns that policy and should enable only its
target's backend: Vulkan on Linux, Metal on macOS, or DX12 on Windows. The
native and WebGPU galleries select exactly their own backend as dev-only host
dependencies; Poolrooms therefore cannot smuggle alien platform machinery into
an application's binary.

The submerged mosaic is registered to the viewport by default. Applications
whose own geometry names the visible tile lattice can register the same floor
to one grout intersection and square pitch, both in egui logical points:

```rust
use brass_poolrooms::water::{Floor, FloorRegistration};

let floor = Floor::shallow(basin)
    .registered(FloorRegistration::square(board.min, cell_pitch));
surface.set_floor(Some(floor));
```

Registration changes only the mosaic's geometry. Poolrooms retains its
material, mortar, deterministic tile variation, and refractive law.

[`Rail`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/struct.Rail.html
[`DateSpool`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/struct.DateSpool.html
[`Checkbox`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/struct.Checkbox.html
[`Monoglyph`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/struct.Monoglyph.html
[`CornerClose`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/struct.CornerClose.html
[`DragHandle`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/struct.DragHandle.html
[`ForgePin`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/struct.ForgePin.html
[`NumberInput`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/struct.NumberInput.html
[`Section`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/struct.Section.html
[`MnemonicText`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/struct.MnemonicText.html
[`Keycap`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/struct.Keycap.html
[`MechanismSize`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/enum.MechanismSize.html
[`Symbol`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/enum.Symbol.html
[`CouplingGap`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/struct.CouplingGap.html
[`Coupled::horizontal`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/struct.Coupled.html#method.horizontal
[`Coupled::horizontal_with_gap`]: https://docs.rs/brass_poolrooms/latest/brass_poolrooms/chrome/struct.Coupled.html#method.horizontal_with_gap
