//! Headless web witnesses rendered through the public Poolrooms egui surface.

use anyhow::{Context as _, Result, bail};
use brass_poolrooms::{
    chrome::{self, MechanismSize, Monoglyph},
    egui::{
        self, Color32, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Stroke, StrokeKind,
        Vec2,
    },
    egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor},
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const GLYPH_SIDE: u32 = 32;

fn main() -> Result<()> {
    let mut args = env::args_os().skip(1).map(PathBuf::from);
    let output = args.next().context("missing output directory")?;
    if args.next().is_some() {
        bail!("usage: brass-poolrooms-web-chrome-forge OUTPUT_DIR");
    }
    fs::create_dir_all(&output).context("create chrome forge output directory")?;
    pollster::block_on(forge(&output))
}

async fn forge(output: &Path) -> Result<()> {
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
    let info = adapter.get_info();
    if info.device_type != wgpu::DeviceType::Cpu {
        bail!(
            "chrome forge requires a CPU GL adapter, found {} ({:?})",
            info.name,
            info.device_type
        );
    }
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("poolrooms-web-chrome-forge"),
            ..Default::default()
        })
        .await
        .context("request chrome forge device")?;
    let ctx = egui::Context::default();
    ctx.set_pixels_per_point(1.0);
    chrome::install(&ctx);
    let renderer = Renderer::new(&device, FORMAT, RendererOptions::PREDICTABLE);
    let mut forge = Forge {
        ctx,
        device,
        queue,
        renderer,
        time: 0.0,
    };

    for (name, size, fill, edge) in [
        (
            "frame-compact.png",
            [357, 110],
            chrome::RAISED,
            chrome::EDGE,
        ),
        (
            "frame-feature.png",
            [720, 132],
            chrome::SURFACE,
            chrome::EDGE,
        ),
        (
            "frame-masthead.png",
            [720, 89],
            chrome::RAISED,
            chrome::EDGE_STRONG,
        ),
    ] {
        let rgba = forge.plate(size, fill, edge)?;
        write_png(&output.join(name), size, &rgba)?;
    }
    let atlas = forge.monoglyph_atlas()?;
    write_png(
        &output.join("monoglyph-atlas.png"),
        [GLYPH_SIDE * 6, GLYPH_SIDE],
        &atlas,
    )
}

struct Forge {
    ctx: egui::Context,
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Renderer,
    time: f64,
}

impl Forge {
    fn plate(&mut self, size: [u32; 2], fill: Color32, edge: Color32) -> Result<Vec<u8>> {
        self.frame(size, Vec::new(), |ui| {
            ui.painter().rect(
                Rect::from_min_size(Pos2::ZERO, Vec2::new(size[0] as f32, size[1] as f32)),
                1,
                fill,
                Stroke::new(1.0, edge),
                StrokeKind::Inside,
            );
        })
    }

    fn monoglyph_atlas(&mut self) -> Result<Vec<u8>> {
        let center = Pos2::new(GLYPH_SIDE as f32 * 0.5, GLYPH_SIDE as f32 * 0.5);
        drop(self.monoglyph_frame(vec![Event::PointerMoved(center)], true)?);
        for _ in 0..3 {
            drop(self.monoglyph_frame(vec![Event::PointerMoved(center)], false)?);
        }
        let rest = self
            .monoglyph_frame(Vec::new(), true)?
            .context("rest frame omitted pixels")?;
        drop(self.monoglyph_frame(
            vec![Event::PointerButton {
                pos: center,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            }],
            false,
        )?);
        for _ in 0..60 {
            drop(self.monoglyph_frame(Vec::new(), false)?);
        }
        let pressed = self
            .monoglyph_frame(Vec::new(), true)?
            .context("pressed frame omitted pixels")?;
        drop(self.monoglyph_frame(
            vec![Event::PointerButton {
                pos: center,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
            false,
        )?);
        let mut samples = Vec::new();
        for frame in 1..=32 {
            if [2, 6, 12, 24].contains(&frame) {
                samples.push(
                    self.monoglyph_frame(Vec::new(), true)?
                        .context("recoil frame omitted pixels")?,
                );
            } else {
                drop(self.monoglyph_frame(Vec::new(), false)?);
            }
        }
        let mut atlas = Vec::with_capacity((GLYPH_SIDE * GLYPH_SIDE * 6 * 4) as usize);
        for row in 0..GLYPH_SIDE as usize {
            let start = row * GLYPH_SIDE as usize * 4;
            let end = start + GLYPH_SIDE as usize * 4;
            atlas.extend_from_slice(&rest[start..end]);
            atlas.extend_from_slice(&pressed[start..end]);
            for sample in &samples {
                atlas.extend_from_slice(&sample[start..end]);
            }
        }
        Ok(atlas)
    }

    fn monoglyph_frame(&mut self, events: Vec<Event>, capture: bool) -> Result<Option<Vec<u8>>> {
        let paint = |ui: &mut egui::Ui| {
            let _response = Monoglyph::new('↗').size(MechanismSize::Large).show(ui);
        };
        if capture {
            self.frame([GLYPH_SIDE, GLYPH_SIDE], events, paint)
                .map(Some)
        } else {
            self.tick([GLYPH_SIDE, GLYPH_SIDE], events, paint);
            Ok(None)
        }
    }

    fn tick(&mut self, size: [u32; 2], events: Vec<Event>, paint: impl FnMut(&mut egui::Ui)) {
        let input = self.input(size, events);
        self.ctx.run_ui(input, paint).drop_without_applying_deltas();
    }

    fn frame(
        &mut self,
        size: [u32; 2],
        events: Vec<Event>,
        paint: impl FnMut(&mut egui::Ui),
    ) -> Result<Vec<u8>> {
        let input = self.input(size, events);
        let output = self.ctx.run_ui(input, paint);
        let primitives = self.ctx.tessellate(output.shapes, output.pixels_per_point);
        let screen = ScreenDescriptor {
            size_in_pixels: size,
            pixels_per_point: output.pixels_per_point,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("poolrooms-web-chrome-target"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[FORMAT],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("poolrooms-web-chrome-render"),
            });
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                self.renderer
                    .update_texture(&self.device, &self.queue, *id, delta);
            }
        }
        let commands = self.renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &primitives,
            &screen,
        );
        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("poolrooms-web-chrome-render"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                })
                .forget_lifetime();
            self.renderer.render(&mut pass, &primitives, &screen);
        }
        let ticket = self
            .queue
            .submit(commands.into_iter().chain([encoder.finish()]));
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(ticket),
                timeout: Some(Duration::from_secs(10)),
            })
            .context("wait for chrome render")?;
        for id in &output.textures_delta.free {
            self.renderer.free_texture(id);
        }
        read_texture(&self.device, &self.queue, &texture, size)
    }

    fn input(&mut self, size: [u32; 2], events: Vec<Event>) -> RawInput {
        self.time += 1.0 / 240.0;
        RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(size[0] as f32, size[1] as f32),
            )),
            time: Some(self.time),
            predicted_dt: 1.0 / 240.0,
            events,
            ..Default::default()
        }
    }
}

fn read_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: [u32; 2],
) -> Result<Vec<u8>> {
    const BYTES_PER_PIXEL: u32 = 4;
    let row = size[0] * BYTES_PER_PIXEL;
    let pitch =
        row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("poolrooms-web-chrome-readback"),
        size: u64::from(pitch) * u64::from(size[1]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("poolrooms-web-chrome-readback"),
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
                rows_per_image: Some(size[1]),
            },
        },
        wgpu::Extent3d {
            width: size[0],
            height: size[1],
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
        .context("wait for chrome readback")?;
    receiver
        .recv_timeout(Duration::from_secs(10))
        .context("receive chrome readback result")?
        .context("map chrome readback")?;
    let mapped = slice
        .get_mapped_range()
        .context("read mapped chrome buffer")?;
    let mut rgba = Vec::with_capacity((row * size[1]) as usize);
    for y in 0..size[1] {
        let start = (y * pitch) as usize;
        rgba.extend_from_slice(&mapped[start..start + row as usize]);
    }
    drop(mapped);
    buffer.unmap();
    Ok(rgba)
}

fn write_png(path: &Path, size: [u32; 2], rgba: &[u8]) -> Result<()> {
    let file = fs::File::create(path).with_context(|| format!("create {}", path.display()))?;
    let mut encoder = png::Encoder::new(file, size[0], size[1]);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45_455));
    let mut writer = encoder.write_header().context("write PNG header")?;
    writer.write_image_data(rgba).context("write PNG pixels")
}
