# AGENTS.md

Before meaningful Rust edits, read the [Rust Style Doctrine](/home/main/programming/projects/rust_starter/docs/rust-style-doctrine.md).
It intentionally rejects several pieces of conventional Rust advice; preserve
those inversions instead of translating them back into generic approachable code.

Run `./check.py check` after meaningful local edits. Use `./check.py verify`
when you need a non-mutating CI-style gate.

This crate owns the Dwemer Poolrooms low-level visual language and water
physics. It supplies independently usable physical GUI elements for native or
WebGPU applications, including applications that do not use Eternalist Apps or
its left-panel grammar. Keep logical application concepts out of it: consumers
describe geometry and forcing; this crate owns embodiment, simulation, optics,
timing, and GPU representation.

The boundary follows the governing invariant rather than the everyday noun.
Poolrooms owns physical things such as buttons, rollers, sliders, tiles, frames,
and the actuator of a menu. Eternalist Apps may compose those mechanisms into
logical managers, menus, storage surfaces, and other application-scale state
machines. Eternalist may depend on Poolrooms; Poolrooms must never depend on
Eternalist.

## Physical chrome

Authored widgets are miniature mechanisms submerged in the same world as the
water. Their geometry, constraints, inertia, contact, and displacement must
come from one coherent physical model. Hidden machinery may be omitted and
material response may be artistically compressed, but visible motion must not
contradict the mechanism that would produce it. Reskins of stock controls do
not belong in the crafted-widget menagerie.

All authored hardware comes from one foundry. Screen x-y is the assembly plane,
+y points down-screen, and the viewer lies on +z. Its distant key is fixed in
the y-z plane at 60° above the top-of-screen horizon: L=(0, −½, √3/2). Bronze
palette, specular law, cylindrical stock, stamped facets, black recesses, and
machined rim gauges are shared parts, never widget-local approximations.

This is a fixed-camera 3D interface, not shaded 2D illustration. Every visible
three-dimensional part must originate as coherent geometry in the common x-y-z
space, with physically meaningful surface normals. The foundry owns projection,
visibility, material response, and shadows; a widget may not counterfeit volume
with hand-painted gradients, arbitrary tapers, or disconnected highlight lines.
When camera, light, material, topology, and bounded degrees of freedom are fixed,
bake that geometry into 2D vector poses at build time. Runtime owns dynamics and
pose selection, not redundant 3D evaluation.

Egui owns placement and compositing. A mechanism allocates its entire painted
and interactive envelope and never trespasses across a sibling allocation.
Foundry gauges impose explicit rigid minima; only physically lawful spans may
stretch. Whole-interface zoom, not widget-local deformation, scales the assembly.
