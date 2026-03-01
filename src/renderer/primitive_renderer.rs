//! Primitive renderer - converts primitives to GPU geometry.
//!
//! This module takes a RenderList of primitives and produces vertex/index
//! buffers ready for GPU rendering.

use crate::renderer::glyph_cache::GlyphAtlas;
use crate::renderer::layout::EditorLayout;
use crate::ui::primitives::{Point, Primitive, Rect, RenderList};

/// Vertex for color/shape rendering (solid colors, borders, etc.)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl ColorVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ColorVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Vertex for text rendering (with UV coordinates for glyph atlas)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl TextVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Geometry data ready for GPU upload
#[derive(Debug, Default)]
pub struct ColorGeometry {
    pub vertices: Vec<ColorVertex>,
    pub indices: Vec<u32>,
}

impl ColorGeometry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(vertex_cap: usize, index_cap: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_cap),
            indices: Vec::with_capacity(index_cap),
        }
    }

    /// Add a quad (rectangle) to the geometry
    pub fn add_quad(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: [f32; 4]) {
        let base = self.vertices.len() as u32;

        self.vertices.push(ColorVertex {
            position: [x1, y1],
            color,
        });
        self.vertices.push(ColorVertex {
            position: [x2, y1],
            color,
        });
        self.vertices.push(ColorVertex {
            position: [x2, y2],
            color,
        });
        self.vertices.push(ColorVertex {
            position: [x1, y2],
            color,
        });

        // Two triangles: 0-1-2 and 0-2-3
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// Add a line as a thin quad
    pub fn add_line(
        &mut self,
        start: Point,
        end: Point,
        width: f32,
        color: [f32; 4],
        layout: &EditorLayout,
    ) {
        // Calculate perpendicular direction for line width
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let len = (dx * dx + dy * dy).sqrt();

        if len < 0.001 {
            return;
        }

        let nx = -dy / len * width * 0.5;
        let ny = dx / len * width * 0.5;

        let [p1x, p1y] = layout.pixel_to_ndc(start.x + nx, start.y + ny);
        let [p2x, p2y] = layout.pixel_to_ndc(start.x - nx, start.y - ny);
        let [p3x, p3y] = layout.pixel_to_ndc(end.x - nx, end.y - ny);
        let [p4x, p4y] = layout.pixel_to_ndc(end.x + nx, end.y + ny);

        let base = self.vertices.len() as u32;
        self.vertices.push(ColorVertex {
            position: [p1x, p1y],
            color,
        });
        self.vertices.push(ColorVertex {
            position: [p4x, p4y],
            color,
        });
        self.vertices.push(ColorVertex {
            position: [p3x, p3y],
            color,
        });
        self.vertices.push(ColorVertex {
            position: [p2x, p2y],
            color,
        });

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// Add a border (4 lines forming a rectangle outline)
    pub fn add_border(&mut self, bounds: Rect, width: f32, color: [f32; 4], layout: &EditorLayout) {
        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.width;
        let h = bounds.height;

        // Top border
        let [x1, y1] = layout.pixel_to_ndc(x, y);
        let [x2, y2] = layout.pixel_to_ndc(x + w, y + width);
        self.add_quad(x1, y1, x2, y2, color);

        // Bottom border
        let [x1, y1] = layout.pixel_to_ndc(x, y + h - width);
        let [x2, y2] = layout.pixel_to_ndc(x + w, y + h);
        self.add_quad(x1, y1, x2, y2, color);

        // Left border
        let [x1, y1] = layout.pixel_to_ndc(x, y + width);
        let [x2, y2] = layout.pixel_to_ndc(x + width, y + h - width);
        self.add_quad(x1, y1, x2, y2, color);

        // Right border
        let [x1, y1] = layout.pixel_to_ndc(x + w - width, y + width);
        let [x2, y2] = layout.pixel_to_ndc(x + w, y + h - width);
        self.add_quad(x1, y1, x2, y2, color);
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

/// Geometry data for text rendering
#[derive(Debug, Default)]
pub struct TextGeometry {
    pub vertices: Vec<TextVertex>,
    pub indices: Vec<u32>,
}

impl TextGeometry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(vertex_cap: usize, index_cap: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_cap),
            indices: Vec::with_capacity(index_cap),
        }
    }

    /// Add a textured quad for a glyph
    pub fn add_glyph_quad(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        uv_min_x: f32,
        uv_min_y: f32,
        uv_max_x: f32,
        uv_max_y: f32,
        color: [f32; 4],
    ) {
        let base = self.vertices.len() as u32;

        self.vertices.push(TextVertex {
            position: [x1, y1],
            uv: [uv_min_x, uv_min_y],
            color,
        });
        self.vertices.push(TextVertex {
            position: [x2, y1],
            uv: [uv_max_x, uv_min_y],
            color,
        });
        self.vertices.push(TextVertex {
            position: [x2, y2],
            uv: [uv_max_x, uv_max_y],
            color,
        });
        self.vertices.push(TextVertex {
            position: [x1, y2],
            uv: [uv_min_x, uv_max_y],
            color,
        });

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

/// The primitive renderer converts primitives into GPU-ready geometry
pub struct PrimitiveRenderer {
    /// Cached geometry for shapes/colors
    pub color_geometry: ColorGeometry,
    /// Cached geometry for text
    pub text_geometry: TextGeometry,
}

impl PrimitiveRenderer {
    pub fn new() -> Self {
        Self {
            color_geometry: ColorGeometry::new(),
            text_geometry: TextGeometry::new(),
        }
    }

    /// Clear all cached geometry
    pub fn clear(&mut self) {
        self.color_geometry.vertices.clear();
        self.color_geometry.indices.clear();
        self.text_geometry.vertices.clear();
        self.text_geometry.indices.clear();
    }

    /// Render a RenderList into GPU geometry
    pub fn render(
        &mut self,
        render_list: &RenderList,
        layout: &EditorLayout,
        glyph_atlas: &mut GlyphAtlas,
    ) {
        self.clear();

        // Sort and partition primitives
        let (shapes, text) = render_list.partition();

        // Render shapes
        for primitive in shapes {
            self.render_shape(primitive, layout);
        }

        // Render text
        for primitive in text {
            self.render_text_primitive(primitive, layout, glyph_atlas);
        }
    }

    fn render_shape(&mut self, primitive: &Primitive, layout: &EditorLayout) {
        match primitive {
            Primitive::Rect { bounds, color, .. } => {
                let [x1, y1] = layout.pixel_to_ndc(bounds.x, bounds.y);
                let [x2, y2] = layout.pixel_to_ndc(bounds.right(), bounds.bottom());
                self.color_geometry.add_quad(x1, y1, x2, y2, *color);
            }

            Primitive::RoundedRect {
                bounds,
                color,
                radius,
                ..
            } => {
                // For now, render rounded rects as regular rects
                // TODO: Implement proper rounded rect rendering with tessellation or SDF
                let effective_radius = radius.min(bounds.width / 2.0).min(bounds.height / 2.0);

                if effective_radius < 1.0 {
                    // Just render as regular rect
                    let [x1, y1] = layout.pixel_to_ndc(bounds.x, bounds.y);
                    let [x2, y2] = layout.pixel_to_ndc(bounds.right(), bounds.bottom());
                    self.color_geometry.add_quad(x1, y1, x2, y2, *color);
                } else {
                    // Tessellate rounded rectangle
                    self.render_rounded_rect(*bounds, *color, effective_radius, layout);
                }
            }

            Primitive::Border {
                bounds,
                color,
                width,
                radius,
                ..
            } => {
                if *radius < 1.0 {
                    self.color_geometry
                        .add_border(*bounds, *width, *color, layout);
                } else {
                    // For rounded borders, render as a rounded rect outline
                    self.render_rounded_border(*bounds, *color, *width, *radius, layout);
                }
            }

            Primitive::Line {
                start,
                end,
                color,
                width,
                ..
            } => {
                self.color_geometry
                    .add_line(*start, *end, *width, *color, layout);
            }

            _ => {} // Text primitives handled separately
        }
    }

    fn render_rounded_rect(
        &mut self,
        bounds: Rect,
        color: [f32; 4],
        radius: f32,
        layout: &EditorLayout,
    ) {
        // Simple tessellation: draw the center cross plus 4 corner arcs
        let segments = 4; // Number of segments per corner arc

        // Center horizontal rect (full width, minus corner height)
        let [x1, y1] = layout.pixel_to_ndc(bounds.x, bounds.y + radius);
        let [x2, y2] = layout.pixel_to_ndc(bounds.right(), bounds.bottom() - radius);
        self.color_geometry.add_quad(x1, y1, x2, y2, color);

        // Top rect (between corners)
        let [x1, y1] = layout.pixel_to_ndc(bounds.x + radius, bounds.y);
        let [x2, y2] = layout.pixel_to_ndc(bounds.right() - radius, bounds.y + radius);
        self.color_geometry.add_quad(x1, y1, x2, y2, color);

        // Bottom rect (between corners)
        let [x1, y1] = layout.pixel_to_ndc(bounds.x + radius, bounds.bottom() - radius);
        let [x2, y2] = layout.pixel_to_ndc(bounds.right() - radius, bounds.bottom());
        self.color_geometry.add_quad(x1, y1, x2, y2, color);

        // Draw corners as triangle fans
        let corners = [
            (
                bounds.x + radius,
                bounds.y + radius,
                std::f32::consts::PI,
                std::f32::consts::FRAC_PI_2 * 3.0,
            ), // Top-left
            (
                bounds.right() - radius,
                bounds.y + radius,
                std::f32::consts::FRAC_PI_2 * 3.0,
                std::f32::consts::TAU,
            ), // Top-right
            (
                bounds.right() - radius,
                bounds.bottom() - radius,
                0.0,
                std::f32::consts::FRAC_PI_2,
            ), // Bottom-right
            (
                bounds.x + radius,
                bounds.bottom() - radius,
                std::f32::consts::FRAC_PI_2,
                std::f32::consts::PI,
            ), // Bottom-left
        ];

        for (cx, cy, start_angle, end_angle) in corners {
            let [center_x, center_y] = layout.pixel_to_ndc(cx, cy);
            let base = self.color_geometry.vertices.len() as u32;

            // Center vertex
            self.color_geometry.vertices.push(ColorVertex {
                position: [center_x, center_y],
                color,
            });

            // Arc vertices
            for i in 0..=segments {
                let angle = start_angle + (end_angle - start_angle) * (i as f32 / segments as f32);
                let px = cx + radius * angle.cos();
                let py = cy + radius * angle.sin();
                let [nx, ny] = layout.pixel_to_ndc(px, py);
                self.color_geometry.vertices.push(ColorVertex {
                    position: [nx, ny],
                    color,
                });
            }

            // Triangle fan indices
            for i in 0..segments {
                self.color_geometry.indices.extend_from_slice(&[
                    base,
                    base + 1 + i as u32,
                    base + 2 + i as u32,
                ]);
            }
        }
    }

    fn render_rounded_border(
        &mut self,
        bounds: Rect,
        color: [f32; 4],
        width: f32,
        radius: f32,
        layout: &EditorLayout,
    ) {
        // Render outer rounded rect and subtract inner (by just drawing the border pieces)
        let outer = bounds;
        let inner = bounds.inset(width);
        let inner_radius = (radius - width).max(0.0);

        // For simplicity, render as 4 border segments plus 4 corner arcs
        // Top segment
        let [x1, y1] = layout.pixel_to_ndc(outer.x + radius, outer.y);
        let [x2, y2] = layout.pixel_to_ndc(outer.right() - radius, outer.y + width);
        self.color_geometry.add_quad(x1, y1, x2, y2, color);

        // Bottom segment
        let [x1, y1] = layout.pixel_to_ndc(outer.x + radius, outer.bottom() - width);
        let [x2, y2] = layout.pixel_to_ndc(outer.right() - radius, outer.bottom());
        self.color_geometry.add_quad(x1, y1, x2, y2, color);

        // Left segment
        let [x1, y1] = layout.pixel_to_ndc(outer.x, outer.y + radius);
        let [x2, y2] = layout.pixel_to_ndc(outer.x + width, outer.bottom() - radius);
        self.color_geometry.add_quad(x1, y1, x2, y2, color);

        // Right segment
        let [x1, y1] = layout.pixel_to_ndc(outer.right() - width, outer.y + radius);
        let [x2, y2] = layout.pixel_to_ndc(outer.right(), outer.bottom() - radius);
        self.color_geometry.add_quad(x1, y1, x2, y2, color);

        // Corner arcs (as thick arcs)
        let segments = 4;
        let corners = [
            (
                outer.x + radius,
                outer.y + radius,
                inner.x + inner_radius,
                inner.y + inner_radius,
                std::f32::consts::PI,
                std::f32::consts::FRAC_PI_2 * 3.0,
            ),
            (
                outer.right() - radius,
                outer.y + radius,
                inner.right() - inner_radius,
                inner.y + inner_radius,
                std::f32::consts::FRAC_PI_2 * 3.0,
                std::f32::consts::TAU,
            ),
            (
                outer.right() - radius,
                outer.bottom() - radius,
                inner.right() - inner_radius,
                inner.bottom() - inner_radius,
                0.0,
                std::f32::consts::FRAC_PI_2,
            ),
            (
                outer.x + radius,
                outer.bottom() - radius,
                inner.x + inner_radius,
                inner.bottom() - inner_radius,
                std::f32::consts::FRAC_PI_2,
                std::f32::consts::PI,
            ),
        ];

        for (ocx, ocy, icx, icy, start_angle, end_angle) in corners {
            for i in 0..segments {
                let a1 = start_angle + (end_angle - start_angle) * (i as f32 / segments as f32);
                let a2 =
                    start_angle + (end_angle - start_angle) * ((i + 1) as f32 / segments as f32);

                let outer1 = layout.pixel_to_ndc(ocx + radius * a1.cos(), ocy + radius * a1.sin());
                let outer2 = layout.pixel_to_ndc(ocx + radius * a2.cos(), ocy + radius * a2.sin());
                let inner1 = layout
                    .pixel_to_ndc(icx + inner_radius * a1.cos(), icy + inner_radius * a1.sin());
                let inner2 = layout
                    .pixel_to_ndc(icx + inner_radius * a2.cos(), icy + inner_radius * a2.sin());

                let base = self.color_geometry.vertices.len() as u32;
                self.color_geometry.vertices.push(ColorVertex {
                    position: outer1,
                    color,
                });
                self.color_geometry.vertices.push(ColorVertex {
                    position: outer2,
                    color,
                });
                self.color_geometry.vertices.push(ColorVertex {
                    position: inner2,
                    color,
                });
                self.color_geometry.vertices.push(ColorVertex {
                    position: inner1,
                    color,
                });

                self.color_geometry.indices.extend_from_slice(&[
                    base,
                    base + 1,
                    base + 2,
                    base,
                    base + 2,
                    base + 3,
                ]);
            }
        }
    }

    fn render_text_primitive(
        &mut self,
        primitive: &Primitive,
        layout: &EditorLayout,
        glyph_atlas: &mut GlyphAtlas,
    ) {
        match primitive {
            Primitive::Text {
                text,
                position,
                color,
                ..
            } => {
                let mut x = position.x;
                let ascent = glyph_atlas.ascent();

                for ch in text.chars() {
                    if let Ok(entry) = glyph_atlas.get_or_rasterize(ch) {
                        if entry.width > 0 && entry.height > 0 {
                            let glyph_x = x + entry.metrics.xmin as f32;
                            let glyph_y = position.y + ascent
                                - entry.metrics.ymin as f32
                                - entry.height as f32;

                            let [x1, y1] = layout.pixel_to_ndc(glyph_x, glyph_y);
                            let [x2, y2] = layout.pixel_to_ndc(
                                glyph_x + entry.width as f32,
                                glyph_y + entry.height as f32,
                            );

                            self.text_geometry.add_glyph_quad(
                                x1,
                                y1,
                                x2,
                                y2,
                                entry.uv_min_x,
                                entry.uv_min_y,
                                entry.uv_max_x,
                                entry.uv_max_y,
                                *color,
                            );
                        }
                        x += entry.metrics.advance_width;
                    }
                }
            }

            Primitive::Glyph {
                char,
                position,
                color,
                ..
            } => {
                let ascent = glyph_atlas.ascent();
                if let Ok(entry) = glyph_atlas.get_or_rasterize(*char) {
                    if entry.width > 0 && entry.height > 0 {
                        let glyph_x = position.x + entry.metrics.xmin as f32;
                        let glyph_y =
                            position.y + ascent - entry.metrics.ymin as f32 - entry.height as f32;

                        let [x1, y1] = layout.pixel_to_ndc(glyph_x, glyph_y);
                        let [x2, y2] = layout.pixel_to_ndc(
                            glyph_x + entry.width as f32,
                            glyph_y + entry.height as f32,
                        );

                        self.text_geometry.add_glyph_quad(
                            x1,
                            y1,
                            x2,
                            y2,
                            entry.uv_min_x,
                            entry.uv_min_y,
                            entry.uv_max_x,
                            entry.uv_max_y,
                            *color,
                        );
                    }
                }
            }

            _ => {} // Shape primitives handled separately
        }
    }
}

impl Default for PrimitiveRenderer {
    fn default() -> Self {
        Self::new()
    }
}
