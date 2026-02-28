use std::sync::Arc;
use winit::window::Window;
use crate::renderer::glyph_cache::GlyphAtlas;
use wgpu::util::DeviceExt;

use super::init::State;

impl State {
                pub fn resize(&mut self, width: u32, height: u32) {
                    if width > 0 && height > 0 {
                        self.config.width = width;
                        self.config.height = height;
                        self.surface.configure(&self.device, &self.config);
                        self.scroll_visual_offset = 0;
                    }
                }

                pub fn set_scale_factor(&mut self, scale_factor: f64) -> anyhow::Result<()> {
                    let new_scale = scale_factor as f32;
                    if (new_scale - self.scale_factor).abs() < 0.001 {
                        return Ok(());
                    }

                    self.scale_factor = new_scale;
                    self.scaled_font_size = crate::state::BASE_FONT_SIZE * new_scale;
                    self.scroll_visual_offset = 0;

                    // Recreate glyph atlas with new font size
                    let font_data = super::font::load_system_font()?;
                    let glyph_atlas = crate::renderer::glyph_cache::GlyphAtlas::new(
                        &font_data,
                        self.scaled_font_size,
                        super::GLYPH_ATLAS_SIZE,
                        super::GLYPH_ATLAS_SIZE,
                    )
                    .map_err(|e| anyhow::anyhow!("{}", e))?;

                    // Upload new atlas data
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
                                bytes_per_row: Some(super::GLYPH_ATLAS_SIZE),
                                rows_per_image: Some(super::GLYPH_ATLAS_SIZE),
                            },
                            wgpu::Extent3d {
                                width: super::GLYPH_ATLAS_SIZE,
                                height: super::GLYPH_ATLAS_SIZE,
                                depth_or_array_layers: 1,
                            },
                        );
                    }

                    self.glyph_atlas = Some(glyph_atlas);
                    Ok(())
                }

                pub fn window(&self) -> &std::sync::Arc<winit::window::Window> {
                    &self.window
                }

                pub fn get_char_at_position(
                    &self,
                    x: f64,
                    y: f64,
                    buffer: &crate::editor::Buffer,
                    show_line_numbers: bool,
                    show_status_bar: bool,
                ) -> Option<(usize, usize)> {
                    let size = self.window.inner_size();
                    let layout = crate::renderer::layout::EditorLayout::new(
                        size.width as f32,
                        size.height as f32,
                        self.scaled_font_size,
                        self.scale_factor,
                        show_line_numbers,
                        show_status_bar,
                    );

                    if let Some(glyph_atlas) = &self.glyph_atlas {
                        let mut atlas_clone = glyph_atlas.clone();
                        layout.hit_test(
                            x as f32,
                            y as f32,
                            buffer,
                            &mut atlas_clone,
                            self.scroll_visual_offset,
                        )
                    } else {
                        None
                    }
                }
            pub fn update_geometry(
                &mut self,
                buffer: &crate::editor::Buffer,
                cursor: &crate::editor::Cursor,
                show_line_numbers: bool,
                show_status_bar: bool,
                status_bar_override: Option<&str>,
                file_path: Option<&str>,
            ) -> anyhow::Result<()> {
                let size = self.window.inner_size();
                let layout = crate::renderer::layout::EditorLayout::new(
                    size.width as f32,
                    size.height as f32,
                    self.scaled_font_size,
                    self.scale_factor,
                    show_line_numbers,
                    show_status_bar,
                );

                // Update UI backgrounds
                let ui_bg = crate::renderer::ui_background::UIBackgroundGeometry::build(&layout);
                if !ui_bg.vertices.is_empty() {
                    self.ui_bg_vertex_buffer = Some(self.device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("ui bg vertex buffer"),
                            contents: bytemuck::cast_slice(&ui_bg.vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        },
                    ));
                    self.ui_bg_index_buffer = Some(self.device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("ui bg index buffer"),
                            contents: bytemuck::cast_slice(&ui_bg.indices),
                            usage: wgpu::BufferUsages::INDEX,
                        },
                    ));
                    self.ui_bg_index_count = ui_bg.indices.len() as u32;
                }

                if let Some(glyph_atlas) = &mut self.glyph_atlas {
                    // Compute wrapped text for rendering and cursor positioning
                    let wrapped_text = crate::renderer::text_geometry::WrappedText::wrap_buffer(buffer, glyph_atlas, &layout);

                    // Update scrolling state based on total visual lines
                    self.total_visual_lines = wrapped_text.total_visual_lines;
                    let visible_lines = layout.visible_lines().max(1);
                    if self.total_visual_lines > visible_lines {
                        let max_offset = self.total_visual_lines.saturating_sub(visible_lines);
                        self.scroll_visual_offset = self.scroll_visual_offset.min(max_offset);
                    } else {
                        self.scroll_visual_offset = 0;
                    }

                    // Compute visible logical lines for syntax highlighting.
                    let first_visual = self
                        .scroll_visual_offset
                        .min(wrapped_text.total_visual_lines.saturating_sub(1));
                    let last_visual = (first_visual + visible_lines).min(wrapped_text.total_visual_lines);
                    let mut visible_logical: Vec<usize> = Vec::new();
                    let mut last_seen: Option<usize> = None;
                    for w in &wrapped_text.wrapped_lines {
                        if w.visual_line < first_visual || w.visual_line >= last_visual {
                            continue;
                        }
                        if last_seen != Some(w.logical_line) {
                            visible_logical.push(w.logical_line);
                            last_seen = Some(w.logical_line);
                        }
                    }

                    let line_colors =
                        self.syntax
                            .highlight_visible_lines(buffer, file_path, &visible_logical);

                    // Update text geometry
                    let text_geometry = crate::renderer::text_geometry::TextGeometry::build_from_buffer(
                        buffer,
                        glyph_atlas,
                        &layout,
                        &wrapped_text,
                        self.scroll_visual_offset,
                        Some(&line_colors),
                    )
                    .map_err(|e| anyhow::anyhow!("{}", e))?;

                    if !text_geometry.vertices.is_empty() {
                        self.text_vertex_buffer = Some(self.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("text vertex buffer"),
                                contents: bytemuck::cast_slice(&text_geometry.vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            },
                        ));
                        self.text_index_buffer = Some(self.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("text index buffer"),
                                contents: bytemuck::cast_slice(&text_geometry.indices),
                                usage: wgpu::BufferUsages::INDEX,
                            },
                        ));
                        self.text_index_count = text_geometry.indices.len() as u32;
                    } else {
                        self.text_index_count = 0;
                    }

                    // Update line numbers with wrapped text info and scrolling
                    let total_lines = buffer.len_lines();
                    let line_nums = crate::renderer::line_numbers::LineNumbersGeometry::build_with_wrap(
                        total_lines,
                        glyph_atlas,
                        &layout,
                        &wrapped_text,
                        self.scroll_visual_offset,
                    )
                    .map_err(|e| anyhow::anyhow!("{}", e))?;

                    if !line_nums.vertices.is_empty() {
                        self.line_numbers_vertex_buffer = Some(self.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("line numbers vertex buffer"),
                                contents: bytemuck::cast_slice(&line_nums.vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            },
                        ));
                        self.line_numbers_index_buffer = Some(self.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("line numbers index buffer"),
                                contents: bytemuck::cast_slice(&line_nums.indices),
                                usage: wgpu::BufferUsages::INDEX,
                            },
                        ));
                        self.line_numbers_index_count = line_nums.indices.len() as u32;
                    } else {
                        self.line_numbers_index_count = 0;
                    }

                    // Update status bar (independent of scrolling)
                    let status_bar =
                        crate::renderer::status_bar::StatusBarGeometry::build(cursor, buffer, glyph_atlas, &layout, status_bar_override)
                        .map_err(|e| anyhow::anyhow!("{}", e))?;

                    if !status_bar.vertices.is_empty() {
                        self.status_bar_vertex_buffer = Some(self.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("status bar vertex buffer"),
                                contents: bytemuck::cast_slice(&status_bar.vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            },
                        ));
                        self.status_bar_index_buffer = Some(self.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("status bar index buffer"),
                                contents: bytemuck::cast_slice(&status_bar.indices),
                                usage: wgpu::BufferUsages::INDEX,
                            },
                        ));
                        self.status_bar_index_count = status_bar.indices.len() as u32;
                    } else {
                        self.status_bar_index_count = 0;
                    }

                    // Update atlas texture
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
                                bytes_per_row: Some(super::GLYPH_ATLAS_SIZE),
                                rows_per_image: Some(super::GLYPH_ATLAS_SIZE),
                            },
                            wgpu::Extent3d {
                                width: super::GLYPH_ATLAS_SIZE,
                                height: super::GLYPH_ATLAS_SIZE,
                                depth_or_array_layers: 1,
                            },
                        );
                    }
                }

                // Update cursor, selection, and scrollbar geometry using current scrolling state
                if let Some(glyph_atlas) = &mut self.glyph_atlas {
                    let cursor_geom = crate::renderer::cursor::CursorGeometry::build_with_wrap(
                        cursor,
                        buffer,
                        &layout,
                        glyph_atlas,
                        self.scroll_visual_offset,
                    );
                    if !cursor_geom.vertices.is_empty() {
                        self.cursor_vertex_buffer = Some(self.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("cursor vertex buffer"),
                                contents: bytemuck::cast_slice(&cursor_geom.vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            },
                        ));
                        self.cursor_index_buffer = Some(self.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("cursor index buffer"),
                                contents: bytemuck::cast_slice(&cursor_geom.indices),
                                usage: wgpu::BufferUsages::INDEX,
                            },
                        ));
                        self.cursor_index_count = cursor_geom.indices.len() as u32;
                    } else {
                        self.cursor_index_count = 0;
                    }

                    let visible_lines = layout.visible_lines().max(1);
                    let scrollbar = crate::renderer::scrollbar::ScrollbarGeometry::build(
                        &layout,
                        self.total_visual_lines,
                        visible_lines,
                        self.scroll_visual_offset,
                    );
                    if !scrollbar.vertices.is_empty() {
                        self.scrollbar_vertex_buffer = Some(self.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("scrollbar vertex buffer"),
                                contents: bytemuck::cast_slice(&scrollbar.vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            },
                        ));
                        self.scrollbar_index_buffer = Some(self.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("scrollbar index buffer"),
                                contents: bytemuck::cast_slice(&scrollbar.indices),
                                usage: wgpu::BufferUsages::INDEX,
                            },
                        ));
                        self.scrollbar_index_count = scrollbar.indices.len() as u32;
                    } else {
                        self.scrollbar_index_count = 0;
                    }
                }

                Ok(())
            }
        pub fn create_text_pipeline(
            device: &wgpu::Device,
            format: wgpu::TextureFormat,
            bind_group_layout: &wgpu::BindGroupLayout,
        ) -> wgpu::RenderPipeline {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("text shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "../renderer/text.wgsl"
                ))),
            });

            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("text pipeline layout"),
                bind_group_layouts: &[bind_group_layout],
                push_constant_ranges: &[],
            });

            let vertex_attrs = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];
            let vertex_layout = wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<crate::renderer::text_geometry::TextVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &vertex_attrs,
            };

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("text pipeline"),
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

        pub fn create_color_pipeline(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("color shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "../renderer/cursor.wgsl"
                ))),
            });

            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("color pipeline layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

            let vertex_attrs = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];
            let vertex_layout = wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<crate::renderer::cursor::CursorVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &vertex_attrs,
            };

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("color pipeline"),
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
    pub async fn state_new_impl(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let scaled_font_size = crate::state::BASE_FONT_SIZE * scale_factor;

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
        let config = wgpu::SurfaceConfiguration {
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

        // Load system font with scaled font size
        let font_data = super::font::load_system_font()?;
        let glyph_atlas = GlyphAtlas::new(
            &font_data,
            scaled_font_size,
            crate::state::GLYPH_ATLAS_SIZE,
            crate::state::GLYPH_ATLAS_SIZE,
        )
        .map_err(|e| anyhow::anyhow!("{}", e))?;

        // Create atlas texture
        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glyph atlas texture"),
            size: wgpu::Extent3d {
                width: crate::state::GLYPH_ATLAS_SIZE,
                height: crate::state::GLYPH_ATLAS_SIZE,
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
                bytes_per_row: Some(crate::state::GLYPH_ATLAS_SIZE),
                rows_per_image: Some(crate::state::GLYPH_ATLAS_SIZE),
            },
            wgpu::Extent3d {
                width: crate::state::GLYPH_ATLAS_SIZE,
                height: crate::state::GLYPH_ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
        );

        // Create bind group layout for text rendering
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
                        &atlas_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Create pipelines
        let text_pipeline = State::create_text_pipeline(&device, config.format, &bind_group_layout);
        let color_pipeline = State::create_color_pipeline(&device, config.format);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            window,
            scale_factor,
            scaled_font_size,
            scroll_visual_offset: 0,
            total_visual_lines: 0,
            text_pipeline: Some(text_pipeline),
            color_pipeline: Some(color_pipeline),
            glyph_atlas: Some(glyph_atlas),
            atlas_texture: Some(atlas_texture),
            atlas_bind_group: Some(atlas_bind_group),
            bind_group_layout: Some(bind_group_layout),
            ui_bg_vertex_buffer: None,
            ui_bg_index_buffer: None,
            ui_bg_index_count: 0,
            text_vertex_buffer: None,
            text_index_buffer: None,
            text_index_count: 0,
            line_numbers_vertex_buffer: None,
            line_numbers_index_buffer: None,
            line_numbers_index_count: 0,
            status_bar_vertex_buffer: None,
            status_bar_index_buffer: None,
            status_bar_index_count: 0,
            cursor_vertex_buffer: None,
            cursor_index_buffer: None,
            cursor_index_count: 0,
            scrollbar_vertex_buffer: None,
            scrollbar_index_buffer: None,
            scrollbar_index_count: 0,
            syntax: crate::syntax::SyntaxHighlighter::new(),
        })
    }
}

