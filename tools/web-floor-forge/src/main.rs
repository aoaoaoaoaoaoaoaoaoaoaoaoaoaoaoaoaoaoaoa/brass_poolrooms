//! Canonical density-correct web witnesses forged from the live Poolrooms WGSL.

use anyhow::{Context as _, Result, bail};
use std::{
    env, fs,
    mem::size_of,
    num::NonZeroU64,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};
use wgpu::util::DeviceExt as _;

const CSS_PERIOD: u32 = 252;
const LOGICAL_PITCH: f32 = 42.0;
const UNIFORM_BYTES: usize = 1_712;
const OPTICS_TIDE_OFFSET: usize = 92;
const FLOOR_VITALS_OFFSET: usize = 1_696;
const DENSITIES: [u32; 3] = [1, 2, 3];
const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

const FORGE_ENTRYPOINT: &str = r"

// Web Kit entrypoint. Every material function above remains the exact
// Poolrooms source; this supplies only the named deterministic deep-rest scene.
@fragment
fn web_deep_rest_floor(in: VsOut) -> @location(0) vec4f {
    let span = forcing.floor_vitals.y * 6.0;
    let px = in.uv * vec2f(span);
    let page = vec3f(12.0 / 255.0, 11.0 / 255.0, 9.0 / 255.0);
    let rays = mat3x2f(px, px, px);
    return vec4f(pool_floor(page, rays, 1.0, forcing.floor_vitals.x), 1.0);
}
";

fn main() -> Result<()> {
    let mut args = env::args_os().skip(1).map(PathBuf::from);
    let poolrooms = args.next().context("missing Poolrooms source directory")?;
    let output = args.next().context("missing output directory")?;
    if args.next().is_some() {
        bail!("usage: brass-poolrooms-web-floor-forge POOLROOMS_SOURCE OUTPUT_DIR");
    }

    fs::create_dir_all(&output).context("create floor forge output directory")?;
    let forcing = fs::read_to_string(poolrooms.join("src/water/engine/forcing.wgsl"))
        .context("read Poolrooms forcing treaty")?;
    let composite = fs::read_to_string(poolrooms.join("src/water/engine/composite.wgsl"))
        .context("read Poolrooms optical shader")?;
    let shader = forcing + &composite + FORGE_ENTRYPOINT;
    pollster::block_on(forge(&shader, &output))
}

async fn forge(shader: &str, output: &Path) -> Result<()> {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::GL;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: true,
            compatible_surface: None,
            apply_limit_buckets: false,
        })
        .await
        .context("request deterministic software GL adapter")?;
    let adapter_info = adapter.get_info();
    if adapter_info.device_type != wgpu::DeviceType::Cpu {
        bail!(
            "floor forge requires a CPU GL adapter, found {} ({:?})",
            adapter_info.name,
            adapter_info.device_type
        );
    }
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("poolrooms-web-floor-forge"),
            ..Default::default()
        })
        .await
        .context("request floor forge device")?;

    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("poolrooms-web-floor"),
        source: wgpu::ShaderSource::Wgsl(shader.into()),
    });
    let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("poolrooms-web-floor-uniform"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(UNIFORM_BYTES as u64),
            },
            count: None,
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("poolrooms-web-floor"),
        bind_group_layouts: &[Some(&uniform_layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("poolrooms-web-floor"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("fullscreen"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("web_deep_rest_floor"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: TARGET_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    for density in DENSITIES {
        let bytes = render(&device, &queue, &pipeline, &uniform_layout, density)?;
        write_ppm(
            &output.join(format!("brass-tiles@{density}x.ppm")),
            CSS_PERIOD * density,
            &bytes,
        )?;
    }
    Ok(())
}

fn render(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::RenderPipeline,
    uniform_layout: &wgpu::BindGroupLayout,
    density: u32,
) -> Result<Vec<u8>> {
    let side = CSS_PERIOD * density;
    let mut uniforms = vec![0_u8; UNIFORM_BYTES];
    put_f32(&mut uniforms, OPTICS_TIDE_OFFSET, 0.0);
    put_f32(&mut uniforms, FLOOR_VITALS_OFFSET, 0.68);
    put_f32(
        &mut uniforms,
        FLOOR_VITALS_OFFSET + size_of::<f32>(),
        LOGICAL_PITCH * density as f32,
    );
    let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("poolrooms-web-floor-forcing"),
        contents: &uniforms,
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("poolrooms-web-floor-forcing"),
        layout: uniform_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 3,
            resource: uniform.as_entire_binding(),
        }],
    });
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("poolrooms-web-floor-target"),
        size: wgpu::Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TARGET_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[TARGET_FORMAT],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("poolrooms-web-floor-render"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("poolrooms-web-floor-render"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &binding, &[]);
        pass.draw(0..3, 0..1);
    }
    let ticket = queue.submit([encoder.finish()]);
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(ticket),
            timeout: Some(Duration::from_secs(10)),
        })
        .context("wait for floor render")?;
    read_texture(device, queue, &texture, side)
}

fn read_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    side: u32,
) -> Result<Vec<u8>> {
    const BYTES_PER_PIXEL: u32 = 4;
    let row = side * BYTES_PER_PIXEL;
    let pitch =
        row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("poolrooms-web-floor-readback"),
        size: u64::from(pitch) * u64::from(side),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("poolrooms-web-floor-readback"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(pitch),
                rows_per_image: Some(side),
            },
        },
        wgpu::Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        },
    );
    let ticket = queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        drop(sender.send(result));
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(ticket),
            timeout: Some(Duration::from_secs(10)),
        })
        .context("wait for floor readback")?;
    receiver
        .recv_timeout(Duration::from_secs(10))
        .context("receive floor readback map result")?
        .context("map floor readback")?;

    let mapped = slice
        .get_mapped_range()
        .context("read mapped floor buffer")?;
    let mut rgba = Vec::with_capacity((row * side) as usize);
    for y in 0..side {
        let start = (y * pitch) as usize;
        rgba.extend_from_slice(&mapped[start..start + row as usize]);
    }
    drop(mapped);
    buffer.unmap();
    Ok(rgba)
}

fn put_f32(bytes: &mut [u8], offset: usize, value: f32) {
    bytes[offset..offset + size_of::<f32>()].copy_from_slice(&value.to_le_bytes());
}

fn write_ppm(path: &Path, side: u32, rgba: &[u8]) -> Result<()> {
    let mut ppm = format!("P6\n{side} {side}\n255\n").into_bytes();
    ppm.reserve((side * side * 3) as usize);
    for pixel in rgba.chunks_exact(4) {
        ppm.extend_from_slice(&pixel[..3]);
    }
    fs::write(path, ppm).with_context(|| format!("write {}", path.display()))
}
