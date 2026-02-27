use std::sync::Arc;
use winit::window::Window;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, RenderPipeline, BindGroup};
use wgpu::util::DeviceExt;
use crate::renderer::glyph_cache::GlyphAtlas;
use crate::renderer::text_geometry::TextGeometry;
use crate::editor::Buffer;

const GLYPH_ATLAS_SIZE: u32 = 1024;
const FONT_SIZE: f32 = 16.0;

pub struct State {
    surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    config: SurfaceConfiguration,
    window: Arc<Window>,
    render_pipeline: Option<RenderPipeline>,
    glyph_atlas: Option<GlyphAtlas>,
    atlas_texture: Option<wgpu::Texture>,
    atlas_bind_group: Option<BindGroup>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_count: u32,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // Create surface
        let surface = instance.create_surface(window.clone())?;

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(anyhow::anyhow!("Failed to find suitable GPU adapter"))?;

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("textedit device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // Load system font
        let font_data = Self::load_system_font()?;
        let glyph_atlas = GlyphAtlas::new(&font_data, FONT_SIZE, GLYPH_ATLAS_SIZE, GLYPH_ATLAS_SIZE)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        // Create atlas texture
        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glyph atlas texture"),
            size: wgpu::Extent3d {
                width: GLYPH_ATLAS_SIZE,
                height: GLYPH_ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Upload atlas data to GPU
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            glyph_atlas.atlas_data(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(GLYPH_ATLAS_SIZE),
                rows_per_image: Some(GLYPH_ATLAS_SIZE),
            },
            wgpu::Extent3d {
                width: GLYPH_ATLAS_SIZE,
                height: GLYPH_ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
        );

        // Create render pipeline and bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("text bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("glyph sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create bind group
        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("text bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &atlas_texture.create_view(&wgpu::TextureViewDescriptor::default())
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Create render pipeline
        let render_pipeline = Self::create_render_pipeline(&device, config.format, &bind_group_layout);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            window,
            render_pipeline: Some(render_pipeline),
            glyph_atlas: Some(glyph_atlas),
            atlas_texture: Some(atlas_texture),
            atlas_bind_group: Some(atlas_bind_group),
            vertex_buffer: None,
            index_buffer: None,
            index_count: 0,
        })
    }

    fn load_system_font() -> anyhow::Result<Vec<u8>> {
        // Try to load system monospace font
        #[cfg(target_os = "macos")]
        {
            std::fs::read("/System/Library/Fonts/SFNSMono.ttf")
                .or_else(|_| std::fs::read("/System/Library/Fonts/Supplemental/Andale Mono.ttf"))
                .map_err(|e| anyhow::anyhow!("Failed to load system font: {}", e))
        }
        #[cfg(target_os = "linux")]
        {
            std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf")
                .or_else(|_| std::fs::read("/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf"))
                .map_err(|e| anyhow::anyhow!("Failed to load system font: {}", e))
        }
        #[cfg(target_os = "windows")]
        {
            std::fs::read("C:\\Windows\\Fonts\\consola.ttf")
                .or_else(|_| std::fs::read("C:\\Windows\\Fonts\\cour.ttf"))
                .map_err(|e| anyhow::anyhow!("Failed to load system font: {}", e))
        }
    }

    fn create_render_pipeline(
        device: &Device,
        format: wgpu::TextureFormat,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("renderer/text.wgsl"))),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render pipeline layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_attrs = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<crate::renderer::text_geometry::TextVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attrs,
        };

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_layout],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
        })
    }

    pub fn update_text_geometry(&mut self, buffer: &Buffer) -> anyhow::Result<()> {
        if let Some(glyph_atlas) = &mut self.glyph_atlas {
            let size = self.window.inner_size();
            
            let geometry = TextGeometry::build_from_buffer(
                buffer,
                glyph_atlas,
                FONT_SIZE,
                size.width as f32,
                size.height as f32,
            )
            .map_err(|e| anyhow::anyhow!("{}", e))?;

            // Update vertex buffer
            let vertex_data = bytemuck::cast_slice(&geometry.vertices);
            self.vertex_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("text vertex buffer"),
                contents: vertex_data,
                usage: wgpu::BufferUsages::VERTEX,
            }));

            // Update index buffer
            let index_data = bytemuck::cast_slice(&geometry.indices);
            self.index_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("text index buffer"),
                contents: index_data,
                usage: wgpu::BufferUsages::INDEX,
            }));

            self.index_count = geometry.indices.len() as u32;

            // Update atlas texture if needed
            if let Some(atlas_texture) = &self.atlas_texture {
                self.queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: atlas_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    glyph_atlas.atlas_data(),
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(GLYPH_ATLAS_SIZE),
                        rows_per_image: Some(GLYPH_ATLAS_SIZE),
                    },
                    wgpu::Extent3d {
                        width: GLYPH_ATLAS_SIZE,
                        height: GLYPH_ATLAS_SIZE,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.03,
                            g: 0.03,
                            b: 0.03,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(pipeline) = &self.render_pipeline {
                render_pass.set_pipeline(pipeline);

                if let Some(bind_group) = &self.atlas_bind_group {
                    render_pass.set_bind_group(0, bind_group, &[]);
                }

                if let (Some(vb), Some(ib)) = (&self.vertex_buffer, &self.index_buffer) {
                    render_pass.set_vertex_buffer(0, vb.slice(..));
                    render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..self.index_count, 0, 0..1);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }
}

