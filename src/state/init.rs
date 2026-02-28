use std::sync::Arc;
use winit::window::Window;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, RenderPipeline, BindGroup};
use crate::renderer::glyph_cache::GlyphAtlas;



pub struct State {
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
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
