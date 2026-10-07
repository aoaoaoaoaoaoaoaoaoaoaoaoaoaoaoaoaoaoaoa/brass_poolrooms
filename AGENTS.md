brass_poolrooms: Brass Poolrooms low-level visual language and water physics; independently usable physical GUI for native or WebGPU apps

## gate
- `./check.py check` after edits; `./check.py verify` for non-mutating gate

## invariants
- Logical app concepts stay out: consumers describe geometry and forcing; crate owns embodiment, simulation, optics, timing, GPU representation
- Poolrooms owns physical things, incl. menu actuator; Eternalist Apps composes them into managers, menus, storage surfaces, app state machines
- Eternalist may depend on Poolrooms; Poolrooms never depends on Eternalist or assumes its left-panel grammar
- Widget gallery may publish one Foundry `demonstration` archive for `eternalist.moe/demos/`; product evidence, not another product or host-support claim
- Authored widgets: miniature mechanisms submerged with the water; geometry, constraints, inertia, contact, displacement from one physical model
- Hidden machinery may be omitted, material response compressed; visible motion never contradicts its mechanism; no reskinned stock controls in menagerie
- One foundry for all hardware: screen x-y assembly plane, +y down-screen, viewer on +z; key light fixed 60° above top horizon, L=(0, −½, √3/2)
- Bronze palette, specular law, cylindrical stock, stamped facets, black recesses, machined rim gauges: shared parts, never widget-local copies
- Fixed-camera 3D interface, not shaded 2D: every visible 3D part is coherent geometry in common x-y-z space with meaningful normals
- Foundry owns projection, visibility, material response, shadows; never fake volume with painted gradients, arbitrary tapers, loose highlight lines
- Fixed camera, light, material, topology, bounded DOF → bake geometry into 2D vector poses at build time; runtime owns only dynamics and pose selection
- Egui owns placement and compositing; mechanism allocates its whole painted and interactive envelope, never trespasses on sibling allocation
- Foundry gauges impose rigid minima; only physically valid spans stretch; whole-interface zoom scales assembly, never widget-local deformation

## routes
- Naming visible Poolrooms objects → `docs/glossary.md`; reuse its nouns exactly, never near-synonyms
