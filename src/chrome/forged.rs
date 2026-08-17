//! Runtime transport for meshes compiled by `brass_foundry`.

use egui::{Color32, Pos2, Vec2};

/// One static projected vertex emitted by Brass Foundry.
#[derive(Clone, Copy)]
pub struct ForgedVertex {
    pub(super) position: [f32; 2],
    pub(super) color: [u8; 4],
}

impl ForgedVertex {
    /// Construct a projected vertex for generated foundry output.
    pub const fn new(position: [f32; 2], color: [u8; 4]) -> Self {
        Self { position, color }
    }
}

/// One immutable, allocation-free mesh emitted by Brass Foundry.
#[derive(Clone, Copy)]
pub struct ForgedMesh {
    pub(super) vertices: &'static [ForgedVertex],
    pub(super) indices: &'static [u32],
}

impl ForgedMesh {
    /// Construct a mesh view over generated static data.
    pub const fn new(vertices: &'static [ForgedVertex], indices: &'static [u32]) -> Self {
        Self { vertices, indices }
    }

    /// Stamp this asset into an egui mesh at `origin`.
    ///
    /// Repeated calls batch any number of instances into one draw mesh.
    pub fn stamp(self, mesh: &mut egui::Mesh, origin: Pos2) {
        let base = mesh.vertices.len() as u32;
        mesh.vertices.reserve(self.vertices.len());
        mesh.indices.reserve(self.indices.len());
        for vertex in self.vertices {
            let [x, y] = vertex.position;
            let [r, g, b, a] = vertex.color;
            mesh.colored_vertex(
                origin + Vec2::new(x, y),
                Color32::from_rgba_unmultiplied(r, g, b, a),
            );
        }
        mesh.indices
            .extend(self.indices.iter().map(|index| base + index));
    }
}
