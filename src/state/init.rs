impl State {
    /// Create a new State with all GPU resources initialized
    pub async fn state_new_impl(window: Arc<Window>, editor_config: crate::config::EditorConfig) -> anyhow::Result<Self> {
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
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Load system font with scaled font size
        let font_data = super::font::load_system_font()?;
        let glyph_atlas = crate::renderer::glyph_cache::GlyphAtlas::new(
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
        let text_pipeline = Self::create_text_pipeline(&device, surface_config.format, &bind_group_layout);
        let color_pipeline = Self::create_color_pipeline(&device, surface_config.format);

        let config_clone = editor_config.clone();
        let syntax_theme = editor_config.syntax_theme.clone();
        Ok(Self {
            surface,
            device,
            queue,
            config: config_clone,
            surface_config,
            width: size.width.max(1),
            height: size.height.max(1),
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
            syntax: crate::syntax::SyntaxHighlighter::new(&syntax_theme),
        })
    }
    // Helper to get current UI colors from config
    pub fn ui_colors(&self) -> crate::renderer::layout::Colors {
        self.config.colors()
    }
}
use std::sync::Arc;
use winit::window::Window;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, RenderPipeline, BindGroup};
use crate::renderer::glyph_cache::GlyphAtlas;



pub struct State {
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub config: crate::config::EditorConfig,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub width: u32,
    pub height: u32,
    pub window: Arc<Window>,
    pub scale_factor: f32,
    pub scaled_font_size: f32,
    pub scroll_visual_offset: usize,
    pub total_visual_lines: usize,
    pub text_pipeline: Option<RenderPipeline>,
    pub color_pipeline: Option<RenderPipeline>,
    pub glyph_atlas: Option<GlyphAtlas>,
    pub atlas_texture: Option<wgpu::Texture>,
    pub atlas_bind_group: Option<BindGroup>,
    pub bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub ui_bg_vertex_buffer: Option<wgpu::Buffer>,
    pub ui_bg_index_buffer: Option<wgpu::Buffer>,
    pub ui_bg_index_count: u32,
    pub text_vertex_buffer: Option<wgpu::Buffer>,
    pub text_index_buffer: Option<wgpu::Buffer>,
    pub text_index_count: u32,
    pub line_numbers_vertex_buffer: Option<wgpu::Buffer>,
    pub line_numbers_index_buffer: Option<wgpu::Buffer>,
    pub line_numbers_index_count: u32,
    pub status_bar_vertex_buffer: Option<wgpu::Buffer>,
    pub status_bar_index_buffer: Option<wgpu::Buffer>,
    pub status_bar_index_count: u32,
    pub cursor_vertex_buffer: Option<wgpu::Buffer>,
    pub cursor_index_buffer: Option<wgpu::Buffer>,
    pub cursor_index_count: u32,
    pub scrollbar_vertex_buffer: Option<wgpu::Buffer>,
    pub scrollbar_index_buffer: Option<wgpu::Buffer>,
    pub scrollbar_index_count: u32,
    pub syntax: crate::syntax::SyntaxHighlighter,
}

impl State {
    pub async fn new(window: Arc<Window>, editor_config: crate::config::EditorConfig) -> anyhow::Result<Self> {
        State::state_new_impl(window, editor_config).await
    }
}
