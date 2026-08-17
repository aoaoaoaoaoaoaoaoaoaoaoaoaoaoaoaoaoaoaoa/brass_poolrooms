const DIE_CELL: usize = 28;
const DIE_COUNT: usize = 7;
const OUTPUT_SCALE: usize = 2;
const SUPERSAMPLE: usize = 4;
const RASTER_SCALE: usize = OUTPUT_SCALE * SUPERSAMPLE;

const FACE_HALF: f32 = 11.05;
const DIE_BODY_HALF: f32 = 12.55;
const FACE_Z: f32 = 3.20;
const DIE_BEVEL_DEPTH: f32 = 1.25;
const DIE_BODY_ROOT: f32 = 0.08;
const CROWN_CELLS: usize = 96;
const CAVITY_AZIMUTHS: usize = 24;
const CAVITY_STEP: f32 = 0.31;
const TOOL_SCAR_PITCH: f32 = 1.68;
const TOOL_SCAR_HALF_WIDTH: f32 = 0.19;
const TOOL_SCAR_DEPTH: f32 = 0.042;

#[derive(Clone, Copy)]
enum Die {
    Linux,
    Windows,
    Macos,
    Chromium,
    WebGpu,
    Rust,
    NativeEgui,
}

impl Die {
    const ALL: [Self; DIE_COUNT] = [
        Self::Linux,
        Self::Windows,
        Self::Macos,
        Self::Chromium,
        Self::WebGpu,
        Self::Rust,
        Self::NativeEgui,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Macos => "macos",
            Self::Chromium => "chromium",
            Self::WebGpu => "webgpu",
            Self::Rust => "rust",
            Self::NativeEgui => "native-egui",
        }
    }

    const fn mark_half(self) -> f32 {
        match self {
            Self::Linux => 9.35,
            Self::Windows => 8.75,
            Self::Macos => 9.25,
            Self::Chromium => 9.15,
            Self::WebGpu => 8.85,
            Self::Rust => 9.35,
            Self::NativeEgui => 8.75,
        }
    }

    fn dent(self, signed_distance: f32, x: f32, y: f32) -> f32 {
        let d = signed_distance.max(0.0);
        let h = self.mark_half();
        let (u, v) = (x / h, y / h);
        match self {
            // Tux is too articulated for a uniform punch. Head, body, wing,
            // and foot bowls form its mass; the exact source linework descends
            // through that relief as a second, narrower tooling pass.
            Self::Linux => {
                let body = 1.18 * oval(u, v, 0.0, 0.17, 0.45, 0.61);
                let head = 1.05 * oval(u, v, 0.0, -0.49, 0.32, 0.34);
                let left_wing = 0.78 * oval(u, v, -0.43, 0.08, 0.19, 0.47);
                let right_wing = 0.78 * oval(u, v, 0.43, 0.08, 0.19, 0.47);
                let left_foot = 0.52 * oval(u, v, -0.28, 0.78, 0.25, 0.14);
                let right_foot = 0.52 * oval(u, v, 0.28, 0.78, 0.25, 0.14);
                let mass = body
                    .max(head)
                    .max(left_wing)
                    .max(right_wing)
                    .max(left_foot)
                    .max(right_foot);
                let belly = 0.23 * oval(u, v, 0.0, 0.23, 0.29, 0.42);
                let tooling = 0.78 * smoothstep(d / 0.38);
                (mass - belly + tooling).clamp(0.0, 2.48)
            }
            // The apple is a round-shouldered sinking die rather than a flat
            // extrusion. Distance to its exact contour supplies the changing
            // curvature; the detached leaf receives the same tool radius.
            Self::Macos => 2.48 * (1.0 - (-d / 1.18).exp()),
            // Four separately masked panes descend to planar floors behind
            // narrow, hard chamfers.
            Self::Windows => 1.78 * smoothstep(d / 0.27),
            // Chromium's ring, vanes, and boss share a medium-radius forming
            // tool, keeping the radial assembly legible at badge scale.
            Self::Chromium => 2.04 * (1.0 - (-d / 0.66).exp()),
            // WebGPU is cut as intersecting facets. The slight geometric rake
            // changes their normals without inventing a second material.
            Self::WebGpu => 1.64 * smoothstep(d / 0.29) * (0.94 + 0.06 * (0.58 * u - 0.42 * v)),
            // The gear and internal R carry sub-pixel apertures; a narrow tool
            // nose preserves them instead of rounding the witness into a cog.
            Self::Rust => 1.58 * smoothstep(d / 0.22),
            // The native-window mark is sheet-metal geometry: crisp casing,
            // screen, neck, and foot with one restrained edge break.
            Self::NativeEgui => 1.70 * smoothstep(d / 0.30),
        }
    }

    const fn tool_scar_normal(self) -> Option<[f32; 2]> {
        match self {
            Self::Windows => Some([0.970_295_7, 0.241_921_9]),
            Self::WebGpu => Some([0.866_025_4, 0.5]),
            Self::Rust => Some([0.939_692_6, -0.342_020_15]),
            Self::NativeEgui => Some([0.766_044_44, 0.642_787_64]),
            Self::Linux | Self::Macos | Self::Chromium => None,
        }
    }

    fn tool_scar(self, signed_distance: f32, dent: f32, x: f32, y: f32) -> f32 {
        let Some([normal_x, normal_y]) = self.tool_scar_normal() else {
            return 0.0;
        };
        let floor = smoothstep((dent - 0.96) / 0.42) * smoothstep((signed_distance - 0.34) / 0.58);
        let phase = (x * normal_x + y * normal_y).rem_euclid(TOOL_SCAR_PITCH);
        let distance = (phase - TOOL_SCAR_PITCH * 0.5).abs();
        let incision = 1.0 - smoothstep(distance / TOOL_SCAR_HALF_WIDTH);
        TOOL_SCAR_DEPTH * floor * incision
    }
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn oval(x: f32, y: f32, cx: f32, cy: f32, rx: f32, ry: f32) -> f32 {
    let radius_squared = ((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2);
    (1.0 - radius_squared).max(0.0).powf(0.72)
}

struct ReliefMask {
    signed: Vec<f32>,
    side: usize,
    center: [f32; 2],
    half_extent: f32,
}

impl ReliefMask {
    fn load(path: &Path) -> io::Result<Self> {
        let alpha = std::fs::read(path)?;
        let side = (alpha.len() as f64).sqrt() as usize;
        if side < 2 || side * side != alpha.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "{} is {} bytes; expected a square raw mask",
                    path.display(),
                    alpha.len()
                ),
            ));
        }
        let inside = alpha.iter().map(|value| *value >= 128).collect::<Vec<_>>();
        let to_inside = chamfer_distance(&inside, side, true);
        let to_outside = chamfer_distance(&inside, side, false);
        let signed = inside
            .iter()
            .enumerate()
            .map(|(index, is_inside)| {
                if *is_inside {
                    to_outside[index] - 0.5
                } else {
                    -(to_inside[index] - 0.5)
                }
            })
            .collect();

        let mut min = [side, side];
        let mut max = [0, 0];
        for (index, is_inside) in inside.iter().enumerate() {
            if *is_inside {
                let (x, y) = (index % side, index / side);
                min[0] = min[0].min(x);
                min[1] = min[1].min(y);
                max[0] = max[0].max(x);
                max[1] = max[1].max(y);
            }
        }
        if min[0] > max[0] || min[1] > max[1] {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} contains no die silhouette", path.display()),
            ));
        }
        let center = [
            (min[0] + max[0]) as f32 * 0.5,
            (min[1] + max[1]) as f32 * 0.5,
        ];
        let half_extent = ((max[0] - min[0]).max(max[1] - min[1]) as f32 + 1.0) * 0.5;
        Ok(Self {
            signed,
            side,
            center,
            half_extent,
        })
    }

    fn signed_distance(&self, x: f32, y: f32, mark_half: f32) -> f32 {
        let p = [
            self.center[0] + x / mark_half * self.half_extent,
            self.center[1] + y / mark_half * self.half_extent,
        ];
        if p[0] < 0.0
            || p[1] < 0.0
            || p[0] >= (self.side - 1) as f32
            || p[1] >= (self.side - 1) as f32
        {
            return -mark_half;
        }
        let lo = [p[0].floor() as usize, p[1].floor() as usize];
        let f = [p[0] - lo[0] as f32, p[1] - lo[1] as f32];
        let at = |x: usize, y: usize| self.signed[y * self.side + x];
        let top = lerp(at(lo[0], lo[1]), at(lo[0] + 1, lo[1]), f[0]);
        let bottom = lerp(at(lo[0], lo[1] + 1), at(lo[0] + 1, lo[1] + 1), f[0]);
        lerp(top, bottom, f[1]) * mark_half / self.half_extent
    }
}

fn chamfer_distance(mask: &[bool], side: usize, target: bool) -> Vec<f32> {
    const FAR: f32 = 1_000_000.0;
    const DIAGONAL: f32 = std::f32::consts::SQRT_2;
    let mut distance = mask
        .iter()
        .map(|value| if *value == target { 0.0 } else { FAR })
        .collect::<Vec<_>>();
    let at = |x: usize, y: usize| y * side + x;

    for y in 0..side {
        for x in 0..side {
            let index = at(x, y);
            if x > 0 {
                distance[index] = distance[index].min(distance[at(x - 1, y)] + 1.0);
            }
            if y > 0 {
                distance[index] = distance[index].min(distance[at(x, y - 1)] + 1.0);
                if x > 0 {
                    distance[index] = distance[index].min(distance[at(x - 1, y - 1)] + DIAGONAL);
                }
                if x + 1 < side {
                    distance[index] = distance[index].min(distance[at(x + 1, y - 1)] + DIAGONAL);
                }
            }
        }
    }
    for y in (0..side).rev() {
        for x in (0..side).rev() {
            let index = at(x, y);
            if x + 1 < side {
                distance[index] = distance[index].min(distance[at(x + 1, y)] + 1.0);
            }
            if y + 1 < side {
                distance[index] = distance[index].min(distance[at(x, y + 1)] + 1.0);
                if x > 0 {
                    distance[index] = distance[index].min(distance[at(x - 1, y + 1)] + DIAGONAL);
                }
                if x + 1 < side {
                    distance[index] = distance[index].min(distance[at(x + 1, y + 1)] + DIAGONAL);
                }
            }
        }
    }
    distance
}

fn die_surface(die: Die, mask: &ReliefMask, x: f32, y: f32) -> f32 {
    let signed = mask.signed_distance(x, y, die.mark_half());
    let dent = die.dent(signed, x, y);
    FACE_Z - dent - die.tool_scar(signed, dent, x, y)
}

fn crown_vertex(die: Die, mask: &ReliefMask, x: f32, y: f32, epsilon: f32) -> Vertex {
    let z = die_surface(die, mask, x, y);
    let dz_dx = (die_surface(die, mask, x + epsilon, y) - die_surface(die, mask, x - epsilon, y))
        / (2.0 * epsilon);
    let dz_dy = (die_surface(die, mask, x, y + epsilon) - die_surface(die, mask, x, y - epsilon))
        / (2.0 * epsilon);
    Vertex::new(V3::new(x, y, z), V3::new(-dz_dx, -dz_dy, 1.0).normalized())
}

struct CavityField {
    sky: Vec<f32>,
}

impl CavityField {
    fn bake(die: Die, mask: &ReliefMask) -> Self {
        let side = CROWN_CELLS + 1;
        let step = FACE_HALF * 2.0 / CROWN_CELLS as f32;
        let epsilon = step * 0.36;
        let mut sky = Vec::with_capacity(side * side);
        for y in 0..side {
            for x in 0..side {
                let x = -FACE_HALF + x as f32 * step;
                let y = -FACE_HALF + y as f32 * step;
                sky.push(cavity_sky_visibility(
                    die,
                    mask,
                    crown_vertex(die, mask, x, y, epsilon),
                ));
            }
        }
        Self { sky }
    }

    fn at(&self, position: V3) -> f32 {
        if position.x <= -FACE_HALF
            || position.y <= -FACE_HALF
            || position.x >= FACE_HALF
            || position.y >= FACE_HALF
        {
            return 1.0;
        }
        let grid =
            |coordinate: f32| (coordinate + FACE_HALF) / (FACE_HALF * 2.0) * CROWN_CELLS as f32;
        let [x, y] = [grid(position.x), grid(position.y)];
        let [ix, iy] = [
            (x.floor() as usize).min(CROWN_CELLS - 1),
            (y.floor() as usize).min(CROWN_CELLS - 1),
        ];
        let side = CROWN_CELLS + 1;
        let sample = |x: usize, y: usize| self.sky[y * side + x];
        let top = lerp(sample(ix, iy), sample(ix + 1, iy), x - ix as f32);
        let bottom = lerp(sample(ix, iy + 1), sample(ix + 1, iy + 1), x - ix as f32);
        lerp(top, bottom, y - iy as f32)
    }
}

fn projected_sky_arc(normal: V3, azimuth: f32, lower: f32) -> f32 {
    let (sin, cos) = azimuth.sin_cos();
    let tangent = normal.x * cos + normal.y * sin;
    let first_lit = if tangent < 0.0 {
        (-tangent).atan2(normal.z)
    } else {
        0.0
    };
    let lower = lower.max(first_lit).min(FRAC_PI_2);
    (tangent * (FRAC_PI_2 * 0.5 - lower * 0.5 - (2.0 * lower).sin() * 0.25)
        + normal.z * lower.cos().powi(2) * 0.5)
        .max(0.0)
}

fn cavity_sky_visibility(die: Die, mask: &ReliefMask, vertex: Vertex) -> f32 {
    if FACE_Z - vertex.position.z <= 0.018 {
        return 1.0;
    }
    let mut visible = 0.0;
    let mut entire_sky = 0.0;
    for azimuth in 0..CAVITY_AZIMUTHS {
        let angle = TAU * (azimuth as f32 + 0.5) / CAVITY_AZIMUTHS as f32;
        let (sin, cos) = angle.sin_cos();
        let mut horizon_slope: f32 = 0.0;
        let mut distance = CAVITY_STEP;
        loop {
            let ray = [
                vertex.position.x + cos * distance,
                vertex.position.y + sin * distance,
            ];
            if ray[0].abs() >= FACE_HALF || ray[1].abs() >= FACE_HALF {
                break;
            }
            let rise = die_surface(die, mask, ray[0], ray[1]) - vertex.position.z - 0.012;
            horizon_slope = horizon_slope.max(rise / distance);
            distance += CAVITY_STEP;
        }
        let horizon = horizon_slope.max(0.0).atan();
        visible += projected_sky_arc(vertex.normal, angle, horizon);
        entire_sky += projected_sky_arc(vertex.normal, angle, 0.0);
    }
    visible / entire_sky.max(f32::EPSILON)
}

fn die_plate(die: Die, mask: &ReliefMask) -> Model {
    let mut model = Model::default();
    let step = FACE_HALF * 2.0 / CROWN_CELLS as f32;
    let sample = |x: usize, y: usize| {
        let x = -FACE_HALF + x as f32 * step;
        let y = -FACE_HALF + y as f32 * step;
        let epsilon = step * 0.36;
        crown_vertex(die, mask, x, y, epsilon)
    };
    for y in 0..CROWN_CELLS {
        for x in 0..CROWN_CELLS {
            model.quad([
                sample(x, y),
                sample(x + 1, y),
                sample(x + 1, y + 1),
                sample(x, y + 1),
            ]);
        }
    }
    square_bevel(
        &mut model,
        FACE_Z,
        FACE_HALF,
        DIE_BODY_HALF,
        DIE_BEVEL_DEPTH,
    );
    square_skirt(
        &mut model,
        FACE_Z,
        DIE_BODY_HALF,
        DIE_BEVEL_DEPTH,
        DIE_BODY_ROOT,
    );
    model
}

fn die_key_visibility(position: V3, die: Die, mask: &ReliefMask) -> f32 {
    if position.x.abs() > FACE_HALF || position.y.abs() > FACE_HALF || FACE_Z - position.z <= 0.018
    {
        return 1.0;
    }
    let light = V3::new(0.0, LIGHT_Y, LIGHT_Z);
    let mut distance = 0.045;
    while distance < 4.6 {
        let ray = position + light * distance;
        if ray.y < -FACE_HALF {
            break;
        }
        if ray.z + 0.022 < die_surface(die, mask, ray.x, ray.y) {
            return 0.0;
        }
        distance += 0.055;
    }
    1.0
}

fn oxide_charge(vertex: Vertex, sky_visibility: f32) -> f32 {
    if vertex.position.x.abs() >= FACE_HALF || vertex.position.y.abs() >= FACE_HALF {
        return 0.0;
    }
    let depth = FACE_Z - vertex.position.z;
    let immersion = smoothstep((depth - 0.24) / 0.96);
    let shelter = smoothstep((1.0 - sky_visibility) / 0.42);
    immersion * lerp(0.58, 1.0, shelter)
}

fn antiqued_lit(vertex: Vertex, key_visibility: f32, sky_visibility: f32) -> [u8; 4] {
    let position = [vertex.position.x, vertex.position.y, vertex.position.z];
    let normal = [vertex.normal.x, vertex.normal.y, vertex.normal.z];
    let (diffuse, reflection) = material_terms(position, normal);
    let oxide = oxide_charge(vertex, sky_visibility);
    let attenuation = |loss: f32| 1.0 - oxide * loss;
    let tone = (DARK_AMBIENT * sky_visibility * attenuation(0.55)
        + key_visibility.clamp(0.0, 1.0)
            * (DARK_DIFFUSE_WEIGHT * attenuation(0.48) * diffuse
                + DARK_BROAD_WEIGHT * attenuation(0.30) * reflection.powf(DARK_BROAD_SHINE)
                + DARK_GLINT_WEIGHT * attenuation(0.12) * reflection.powf(DARK_GLINT_SHINE)))
    .min(DARK_TONE_CEILING);
    let rgb = brass_foundry::darkened_bronze_rgb(tone);
    let stain = [attenuation(0.07), attenuation(0.11), attenuation(0.16)];
    let channel = |index: usize| {
        (f32::from(rgb[index]) * stain[index])
            .round()
            .clamp(0.0, 255.0) as u8
    };
    [channel(0), channel(1), channel(2), 255]
}

fn compile_die(model: &Model, die: Die, mask: &ReliefMask, cavity: &CavityField) -> Compiled {
    compile_bronze_with(model, |vertex| {
        antiqued_lit(
            vertex,
            die_key_visibility(vertex.position, die, mask),
            cavity.at(vertex.position),
        )
    })
}

#[derive(Clone, Copy, Default)]
struct Premul {
    rgba: [f32; 4],
}

struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<Premul>,
}

impl Canvas {
    fn new() -> Self {
        let width = DIE_CELL * DIE_COUNT * RASTER_SCALE;
        let height = DIE_CELL * RASTER_SCALE;
        Self {
            width,
            height,
            pixels: vec![Premul::default(); width * height],
        }
    }

    fn paint(&mut self, cell: usize, mesh: &Compiled) {
        let origin = [
            (cell as f32 * DIE_CELL as f32 + DIE_CELL as f32 * 0.5) * RASTER_SCALE as f32,
            DIE_CELL as f32 * 0.5 * RASTER_SCALE as f32,
        ];
        for triangle in mesh.indices().chunks_exact(3) {
            let pixels = [
                mesh.vertices()[triangle[0] as usize],
                mesh.vertices()[triangle[1] as usize],
                mesh.vertices()[triangle[2] as usize],
            ];
            self.triangle(origin, pixels);
        }
    }

    fn triangle(&mut self, origin: [f32; 2], triangle: [Pixel; 3]) {
        let p = triangle.map(|vertex| {
            [
                origin[0] + vertex.position[0] * RASTER_SCALE as f32,
                origin[1] + vertex.position[1] * RASTER_SCALE as f32,
            ]
        });
        let area = edge(p[0], p[1], p[2]);
        if area.abs() < f32::EPSILON {
            return;
        }
        let min_x = p
            .iter()
            .map(|point| point[0])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as usize;
        let max_x = p
            .iter()
            .map(|point| point[0])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min((self.width - 1) as f32) as usize;
        let min_y = p
            .iter()
            .map(|point| point[1])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as usize;
        let max_y = p
            .iter()
            .map(|point| point[1])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min((self.height - 1) as f32) as usize;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let sample = [x as f32 + 0.5, y as f32 + 0.5];
                let barycentric = [
                    edge(p[1], p[2], sample) / area,
                    edge(p[2], p[0], sample) / area,
                    edge(p[0], p[1], sample) / area,
                ];
                if barycentric.iter().any(|weight| *weight < -0.000_02) {
                    continue;
                }
                let mut rgba = [0.0; 4];
                for (vertex, weight) in triangle.iter().zip(barycentric) {
                    for (channel, value) in rgba.iter_mut().zip(vertex.color) {
                        *channel += f32::from(value) / 255.0 * weight;
                    }
                }
                self.over(x, y, rgba);
            }
        }
    }

    fn over(&mut self, x: usize, y: usize, rgba: [f32; 4]) {
        let destination = &mut self.pixels[y * self.width + x].rgba;
        let alpha = rgba[3].clamp(0.0, 1.0);
        for channel in 0..3 {
            destination[channel] =
                rgba[channel].clamp(0.0, 1.0) * alpha + destination[channel] * (1.0 - alpha);
        }
        destination[3] = alpha + destination[3] * (1.0 - alpha);
    }

    fn downsample(&self) -> Vec<u8> {
        let width = DIE_CELL * DIE_COUNT * OUTPUT_SCALE;
        let height = DIE_CELL * OUTPUT_SCALE;
        let mut output = Vec::with_capacity(width * height * 4);
        for y in 0..height {
            for x in 0..width {
                let mut sum = [0.0; 4];
                for sy in 0..SUPERSAMPLE {
                    for sx in 0..SUPERSAMPLE {
                        let sample = self.pixels
                            [(y * SUPERSAMPLE + sy) * self.width + x * SUPERSAMPLE + sx]
                            .rgba;
                        for (total, value) in sum.iter_mut().zip(sample) {
                            *total += value;
                        }
                    }
                }
                let samples = (SUPERSAMPLE * SUPERSAMPLE) as f32;
                sum.iter_mut().for_each(|value| *value /= samples);
                let alpha = sum[3];
                for premultiplied in &sum[..3] {
                    output.push(if alpha > f32::EPSILON {
                        (premultiplied / alpha * 255.0).round().clamp(0.0, 255.0) as u8
                    } else {
                        0
                    });
                }
                output.push((alpha * 255.0).round().clamp(0.0, 255.0) as u8);
            }
        }
        output
    }

    fn write_pam(&self, path: &Path) -> io::Result<()> {
        let width = DIE_CELL * DIE_COUNT * OUTPUT_SCALE;
        let height = DIE_CELL * OUTPUT_SCALE;
        let mut output = BufWriter::new(File::create(path)?);
        write!(
            output,
            "P7\nWIDTH {width}\nHEIGHT {height}\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n"
        )?;
        output.write_all(&self.downsample())
    }
}

fn edge(a: [f32; 2], b: [f32; 2], p: [f32; 2]) -> f32 {
    (p[0] - a[0]) * (b[1] - a[1]) - (p[1] - a[1]) * (b[0] - a[0])
}

pub(crate) fn forge(mask_dir: &Path, output: &Path) -> io::Result<()> {
    verify_geometry();
    let mut canvas = Canvas::new();
    for (index, die) in Die::ALL.into_iter().enumerate() {
        let mask = ReliefMask::load(&mask_dir.join(format!("{}.gray", die.name())))?;
        let model = die_plate(die, &mask);
        let cavity = CavityField::bake(die, &mask);
        canvas.paint(index, &compile_shadow(&model, 0.0, 68));
        canvas.paint(index, &compile_die(&model, die, &mask, &cavity));
    }
    canvas.write_pam(output)
}
