//! Build-time geometry compiler for the Brass Poolrooms visual universe.
//!
//! Applications author three-dimensional [`Model`] dies and forge them into
//! compact, colored two-dimensional triangle meshes. Projection, illumination,
//! material response, visibility, and shadows share the exact laws used by
//! Poolrooms' own mechanisms.

use std::{
    collections::HashMap,
    io::{self, Write},
    ops::{Add, Div, Mul, Sub},
};

/// Oxidized-bronze shadow charge.
pub const BRONZE_SHADOW: [f32; 3] = [34.0, 28.0, 19.0];
/// Oxidized-bronze body charge.
pub const BRONZE_BODY: [f32; 3] = [104.0, 86.0, 58.0];
/// Polished-bronze highlight charge.
pub const BRONZE_GLINT: [f32; 3] = [196.0, 170.0, 124.0];
/// Largest interpolation cell admitted by the dark-bronze reflection lobe.
pub const DARK_REFLECTION_CELL: f32 = 4.0;

/// Down-screen component of the fixed distant illuminant.
pub const LIGHT_Y: f32 = -0.5;
/// Viewer-facing component of the fixed distant illuminant.
pub const LIGHT_Z: f32 = 0.866_025_4;
/// Down-screen component of the fixed halfway vector.
pub const HALF_Y: f32 = -0.258_819_04;
/// Viewer-facing component of the fixed halfway vector.
pub const HALF_Z: f32 = 0.965_925_8;
/// Ordinary bare-metal specular exponent.
pub const METAL_SHINE: f32 = 14.0;
/// Mirror-polished metal specular exponent.
pub const MIRROR_SHINE: f32 = 72.0;
/// Work-darkened metal ambient response.
pub const DARK_AMBIENT: f32 = 0.13;
/// Work-darkened metal diffuse weight.
pub const DARK_DIFFUSE_WEIGHT: f32 = 0.32;
/// Work-darkened metal broad-reflection weight.
pub const DARK_BROAD_WEIGHT: f32 = 0.01;
/// Work-darkened metal broad-reflection exponent.
pub const DARK_BROAD_SHINE: f32 = 12.0;
/// Work-darkened metal glint weight.
pub const DARK_GLINT_WEIGHT: f32 = 3.0;
/// Work-darkened metal glint exponent.
pub const DARK_GLINT_SHINE: f32 = 128.0;
/// Work-darkened metal response ceiling.
pub const DARK_TONE_CEILING: f32 = 0.72;
/// Work-darkened bronze output exposure.
pub const DARK_EXPOSURE: f32 = 1.20;
/// Fixed perspective eye above the assembly plane, in logical points.
pub const EYE_Z: f32 = 80.0;

/// A point, vector, or normal in the common fixed-camera universe.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    /// Screen-horizontal coordinate.
    pub x: f32,
    /// Screen-vertical coordinate, positive down-screen.
    pub y: f32,
    /// Elevation toward the viewer.
    pub z: f32,
}

impl Vec3 {
    /// The additive identity.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    /// Construct a vector in foundry coordinates.
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Return the Euclidean inner product.
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    /// Return the right-handed cross product.
    pub fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }

    /// Return the Euclidean length.
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// Return a unit vector, leaving the zero vector at zero.
    pub fn normalized(self) -> Self {
        self / self.length().max(f32::EPSILON)
    }

    /// Rotate around the assembly-plane normal.
    pub fn rotate_z(self, angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(
            self.x * cos - self.y * sin,
            self.x * sin + self.y * cos,
            self.z,
        )
    }

    /// Expose this vector as an array for scalar material laws.
    pub const fn array(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

/// A model vertex carrying one physically meaningful surface normal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    /// Three-dimensional location.
    pub position: Vec3,
    /// Surface normal at the location.
    pub normal: Vec3,
}

impl Vertex {
    /// Construct a model vertex.
    pub const fn new(position: Vec3, normal: Vec3) -> Self {
        Self { position, normal }
    }
}

/// An application-authored three-dimensional die.
#[derive(Clone, Debug, Default)]
pub struct Model {
    /// Oriented triangle facets.
    pub triangles: Vec<[Vertex; 3]>,
}

impl Model {
    /// Add an oriented triangle.
    pub fn triangle(&mut self, a: Vertex, b: Vertex, c: Vertex) {
        self.triangles.push([a, b, c]);
    }

    /// Add a counter-clockwise quadrilateral as two triangles.
    pub fn quad(&mut self, [a, b, c, d]: [Vertex; 4]) {
        self.triangle(a, b, c);
        self.triangle(a, c, d);
    }

    /// Transfer every facet from another model into this die.
    pub fn append(&mut self, mut rhs: Self) {
        self.triangles.append(&mut rhs.triangles);
    }

    /// Transform positions and normals into a new rigid pose.
    pub fn transformed(
        &self,
        position: impl Fn(Vec3) -> Vec3,
        normal: impl Fn(Vec3) -> Vec3,
    ) -> Self {
        Self {
            triangles: self
                .triangles
                .iter()
                .map(|triangle| {
                    triangle
                        .map(|vertex| Vertex::new(position(vertex.position), normal(vertex.normal)))
                })
                .collect(),
        }
    }
}

/// One projected, illuminated vertex in a forged mesh.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pixel {
    /// Projected position relative to the die origin.
    pub position: [f32; 2],
    /// Straight-alpha sRGBA color.
    pub color: [u8; 4],
}

/// A compact two-dimensional mesh forged from a three-dimensional die.
#[derive(Default)]
pub struct Mesh {
    /// Interned projected vertices.
    vertices: Vec<Pixel>,
    /// Triangle-list indices into `vertices`.
    indices: Vec<u32>,
    intern: HashMap<([u32; 2], [u8; 4]), u32>,
}

/// Rust visibility assigned to one emitted mesh binding.
#[derive(Clone, Copy, Debug)]
pub enum RustReach {
    /// Visible to the module containing the generated module.
    Module,
    /// Visible throughout the consuming crate.
    Crate,
    /// Publicly exported by the consuming crate.
    Public,
}

/// Runtime type names used by generated Rust mesh bindings.
#[derive(Clone, Copy, Debug)]
pub struct RustDialect {
    vertex: &'static str,
    mesh: &'static str,
}

impl RustDialect {
    /// The public runtime transport exported by `brass_poolrooms::chrome`.
    pub const FORGED: Self = Self::new("ForgedVertex", "ForgedMesh");

    /// Name an equivalent runtime mesh transport in the including module.
    pub const fn new(vertex: &'static str, mesh: &'static str) -> Self {
        Self { vertex, mesh }
    }
}

impl RustReach {
    const fn token(self) -> &'static str {
        match self {
            Self::Module => "pub(super)",
            Self::Crate => "pub(crate)",
            Self::Public => "pub",
        }
    }
}

impl Mesh {
    /// Read the projected vertices in painter order.
    pub fn vertices(&self) -> &[Pixel] {
        &self.vertices
    }

    /// Read the triangle-list indices into [`Self::vertices`].
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// Intern a projected vertex and return its index.
    pub fn vertex(&mut self, pixel: Pixel) -> u32 {
        let key = (
            [pixel.position[0].to_bits(), pixel.position[1].to_bits()],
            pixel.color,
        );
        if let Some(index) = self.intern.get(&key) {
            return *index;
        }
        assert!(
            u32::try_from(self.vertices.len()).is_ok(),
            "forged mesh exceeds u32 indexing"
        );
        let index = self.vertices.len() as u32;
        self.vertices.push(pixel);
        let _prior = self.intern.insert(key, index);
        index
    }

    /// Add one projected triangle.
    pub fn triangle(&mut self, pixels: [Pixel; 3]) {
        let indices = pixels.map(|pixel| self.vertex(pixel));
        self.indices.extend(indices);
    }
}

/// Emit one mesh as static Rust data.
///
/// The including module must bring `ForgedVertex` and `ForgedMesh` from
/// `brass_poolrooms::chrome` into scope. The generated representation performs
/// no runtime decoding or allocation.
pub fn emit_rust(
    out: &mut impl Write,
    name: &str,
    mesh: &Mesh,
    reach: RustReach,
) -> io::Result<()> {
    emit_rust_as(out, name, mesh, reach, RustDialect::FORGED)
}

/// Emit one mesh using equivalent consumer-owned runtime type names.
pub fn emit_rust_as(
    out: &mut impl Write,
    name: &str,
    mesh: &Mesh,
    reach: RustReach,
    dialect: RustDialect,
) -> io::Result<()> {
    assert!(
        !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte == b'_' || byte.is_ascii_uppercase() || byte.is_ascii_digit()),
        "forged Rust binding must be an uppercase identifier"
    );
    let RustDialect {
        vertex: vertex_type,
        mesh: mesh_type,
    } = dialect;
    assert!(
        [vertex_type, mesh_type]
            .into_iter()
            .all(valid_rust_type_name),
        "forged Rust dialect must contain bare type identifiers"
    );
    writeln!(out, "static {name}_VERTICES: &[{vertex_type}] = &[")?;
    for vertex in &mesh.vertices {
        let [x, y] = vertex.position;
        let [r, g, b, a] = vertex.color;
        writeln!(
            out,
            "{vertex_type}::new([{}, {}], [{r}, {g}, {b}, {a}]),",
            scalar(x),
            scalar(y)
        )?;
    }
    writeln!(out, "];")?;
    writeln!(out, "static {name}_INDICES: &[u32] = &[")?;
    for chunk in mesh.indices.chunks(24) {
        for index in chunk {
            write!(out, "{index},")?;
        }
        writeln!(out)?;
    }
    writeln!(out, "];")?;
    writeln!(
        out,
        "{} static {name}: {mesh_type} = {mesh_type}::new({name}_VERTICES, {name}_INDICES);",
        reach.token()
    )
}

fn valid_rust_type_name(name: &str) -> bool {
    name.bytes().enumerate().all(|(index, byte)| {
        byte == b'_' || byte.is_ascii_alphabetic() || (index > 0 && byte.is_ascii_digit())
    }) && !name.is_empty()
}

fn scalar(value: f32) -> String {
    let bits = value.to_bits();
    format!("f32::from_bits(0x{:04x}_{:04x})", bits >> 16, bits & 0xffff)
}

/// The three canonical bronze charges.
#[derive(Clone, Copy, Debug)]
pub enum Charge {
    /// Bare machined bronze at the given exposure.
    Bronze(f32),
    /// Work-darkened bronze at the given exposure.
    Darkened(f32),
    /// Mirror-polished ceremonial bronze.
    Polished,
}

/// Forge a die with one canonical material charge.
pub fn forge(model: &Model, charge: Charge) -> Mesh {
    compile_with(model, |vertex| match charge {
        Charge::Bronze(exposure) => lit(vertex, exposure),
        Charge::Darkened(exposure) => darkened_lit(vertex, exposure),
        Charge::Polished => polished_lit(vertex),
    })
}

/// Forge visible facets with an application-supplied material response.
pub fn compile_with(model: &Model, illuminate: impl Fn(Vertex) -> [u8; 4]) -> Mesh {
    let mut facets = model
        .triangles
        .iter()
        .filter(|triangle| visible(triangle))
        .collect::<Vec<_>>();
    facets.sort_by(|a, b| depth(a).total_cmp(&depth(b)));
    let mut compiled = Mesh::default();
    for triangle in facets {
        compiled.triangle(triangle.map(|vertex| Pixel {
            position: project(vertex.position),
            color: illuminate(vertex),
        }));
    }
    compiled
}

/// Forge the directional shadow cast onto one planar receiver.
pub fn shadow(model: &Model, receiver_z: f32, alpha: u8) -> Mesh {
    let light = Vec3::new(0.0, LIGHT_Y, LIGHT_Z);
    let mut compiled = Mesh::default();
    for triangle in &model.triangles {
        let normal = triangle
            .iter()
            .fold(Vec3::ZERO, |sum, vertex| sum + vertex.normal)
            .normalized();
        if normal.dot(light) <= 0.0
            || triangle
                .iter()
                .any(|vertex| vertex.position.z <= receiver_z)
        {
            continue;
        }
        compiled.triangle(triangle.map(|vertex| {
            let distance = (vertex.position.z - receiver_z) / LIGHT_Z;
            Pixel {
                position: project(vertex.position - light * distance),
                color: [0, 0, 0, alpha],
            }
        }));
    }
    compiled
}

/// Forge receiver-independent directional shadow coordinates.
///
/// A runtime receiver at `z=r` applies `s=eye/(eye-r)`, then
/// `(x, y+kz) -> s*(x, y+kz-kr)`.
pub fn shadow_source(model: &Model, receiver_ceiling: f32, alpha: u8) -> Mesh {
    let light = Vec3::new(0.0, LIGHT_Y, LIGHT_Z);
    let slope = -LIGHT_Y / LIGHT_Z;
    let mut compiled = Mesh::default();
    for triangle in &model.triangles {
        let normal = triangle
            .iter()
            .fold(Vec3::ZERO, |sum, vertex| sum + vertex.normal)
            .normalized();
        if normal.dot(light) <= 0.0
            || triangle
                .iter()
                .any(|vertex| vertex.position.z <= receiver_ceiling)
        {
            continue;
        }
        compiled.triangle(triangle.map(|vertex| Pixel {
            position: [
                vertex.position.x,
                vertex.position.y + slope * vertex.position.z,
            ],
            color: [0, 0, 0, alpha],
        }));
    }
    compiled
}

/// Project one foundry-space point into the assembly plane.
pub fn project(point: Vec3) -> [f32; 2] {
    let scale = EYE_Z / (EYE_Z - point.z).max(1.0);
    [point.x * scale, point.y * scale]
}

/// Return whether an oriented facet faces the fixed camera.
pub fn visible(triangle: &[Vertex; 3]) -> bool {
    let center = triangle
        .iter()
        .fold(Vec3::ZERO, |sum, vertex| sum + vertex.position)
        / 3.0;
    let normal = triangle
        .iter()
        .fold(Vec3::ZERO, |sum, vertex| sum + vertex.normal)
        .normalized();
    normal.dot((Vec3::new(0.0, 0.0, EYE_Z) - center).normalized()) > 0.0
}

/// Return the painter-order depth of one facet.
pub fn depth(triangle: &[Vertex; 3]) -> f32 {
    triangle.iter().map(|vertex| vertex.position.z).sum::<f32>() / 3.0
}

/// Illuminate bare bronze.
pub fn lit(vertex: Vertex, exposure: f32) -> [u8; 4] {
    expose(
        bronze_rgb(metal_tone(vertex.position.array(), vertex.normal.array())),
        exposure,
    )
}

/// Illuminate bare bronze with explicit key visibility.
pub fn lit_with_key(vertex: Vertex, visibility: f32) -> [u8; 4] {
    opaque(bronze_rgb(metal_tone_with_key(
        vertex.position.array(),
        vertex.normal.array(),
        visibility,
    )))
}

/// Illuminate work-darkened bronze.
pub fn darkened_lit(vertex: Vertex, exposure: f32) -> [u8; 4] {
    expose(
        darkened_bronze_rgb(darkened_metal_tone(
            vertex.position.array(),
            vertex.normal.array(),
        )),
        exposure,
    )
}

/// Illuminate work-darkened bronze with explicit key visibility.
pub fn darkened_lit_with_key(vertex: Vertex, visibility: f32) -> [u8; 4] {
    opaque(darkened_bronze_rgb(darkened_metal_tone_with_key(
        vertex.position.array(),
        vertex.normal.array(),
        visibility,
    )))
}

/// Illuminate mirror-polished ceremonial bronze.
pub fn polished_lit(vertex: Vertex) -> [u8; 4] {
    opaque(bronze_rgb(polished_metal_tone(
        vertex.position.array(),
        vertex.normal.array(),
    )))
}

fn opaque([r, g, b]: [u8; 3]) -> [u8; 4] {
    [r, g, b, 255]
}

fn expose(rgb: [u8; 3], exposure: f32) -> [u8; 4] {
    let channel = |value: u8| (f32::from(value) * exposure).round().clamp(0.0, 255.0) as u8;
    [channel(rgb[0]), channel(rgb[1]), channel(rgb[2]), 255]
}

/// Interpolate the canonical bronze charge at one illumination tone.
pub fn bronze_rgb(tone: f32) -> [u8; 3] {
    let tone = tone.clamp(0.0, 1.0);
    let (lo, hi, t) = if tone < 0.6 {
        (BRONZE_SHADOW, BRONZE_BODY, tone / 0.6)
    } else {
        (BRONZE_BODY, BRONZE_GLINT, (tone - 0.6) / 0.4)
    };
    let channel = |i: usize| (lo[i] + (hi[i] - lo[i]) * t).round() as u8;
    [channel(0), channel(1), channel(2)]
}

/// Apply the foundry's oxide exposure to a bronze response.
pub fn darkened_bronze_rgb(tone: f32) -> [u8; 3] {
    bronze_rgb(tone).map(|channel| {
        (f32::from(channel) * DARK_EXPOSURE)
            .round()
            .clamp(0.0, 255.0) as u8
    })
}

/// Return bare-bronze material tone at one point and normal.
pub fn metal_tone(position: [f32; 3], normal: [f32; 3]) -> f32 {
    metal_tone_with_key(position, normal, 1.0)
}

/// Return mirror-polished material tone at one point and normal.
pub fn polished_metal_tone(position: [f32; 3], normal: [f32; 3]) -> f32 {
    let (diffuse, reflection) = material_terms(position, normal);
    (0.13 + 0.43 * diffuse + 1.85 * reflection.powf(MIRROR_SHINE)).min(1.0)
}

/// Return work-darkened material tone at one point and normal.
pub fn darkened_metal_tone(position: [f32; 3], normal: [f32; 3]) -> f32 {
    darkened_metal_tone_with_key(position, normal, 1.0)
}

/// Return bare-bronze tone with explicit geometric key visibility.
pub fn metal_tone_with_key(position: [f32; 3], normal: [f32; 3], key_visibility: f32) -> f32 {
    let (diffuse, reflection) = material_terms(position, normal);
    0.16 + key_visibility.clamp(0.0, 1.0) * (0.5 * diffuse + 0.8 * reflection.powf(METAL_SHINE))
}

/// Return work-darkened tone with explicit geometric key visibility.
pub fn darkened_metal_tone_with_key(
    position: [f32; 3],
    normal: [f32; 3],
    key_visibility: f32,
) -> f32 {
    let (diffuse, reflection) = material_terms(position, normal);
    (DARK_AMBIENT
        + key_visibility.clamp(0.0, 1.0)
            * (DARK_DIFFUSE_WEIGHT * diffuse
                + DARK_BROAD_WEIGHT * reflection.powf(DARK_BROAD_SHINE)
                + DARK_GLINT_WEIGHT * reflection.powf(DARK_GLINT_SHINE)))
    .min(DARK_TONE_CEILING)
}

/// Return diffuse and Blinn-Phong reflection terms under the fixed key.
pub fn material_terms(position: [f32; 3], normal: [f32; 3]) -> (f32, f32) {
    let normalize = |[x, y, z]: [f32; 3]| {
        let length = (x * x + y * y + z * z).sqrt().max(f32::EPSILON);
        [x / length, y / length, z / length]
    };
    let [nx, ny, nz] = normalize(normal);
    let [vx, vy, vz] = normalize([-position[0], -position[1], EYE_Z - position[2]]);
    let [hx, hy, hz] = normalize([vx, vy + LIGHT_Y, vz + LIGHT_Z]);
    let diffuse = (ny * LIGHT_Y + nz * LIGHT_Z).max(0.0);
    let reflection = (nx * hx + ny * hy + nz * hz).max(0.0);
    (diffuse, reflection)
}
