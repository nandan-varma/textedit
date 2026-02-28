use super::layout::EditorLayout;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

pub struct UIBackgroundGeometry {
    pub vertices: Vec<ColorVertex>,
    pub indices: Vec<u32>,
}

impl UIBackgroundGeometry {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Build all UI background elements (gutter, status bar, separators)
    pub fn build(layout: &EditorLayout, colors: &super::layout::Colors) -> Self {
        let mut geometry = UIBackgroundGeometry::new();

        // Gutter background
        geometry.add_rect(
            layout,
            layout.gutter.x,
            layout.gutter.y,
            layout.gutter.width,
            layout.gutter.height,
            colors.gutter_background,
        );

        // Gutter separator (1px line)
        geometry.add_rect(
            layout,
            layout.gutter.right() - 1.0,
            layout.gutter.y,
            1.0,
            layout.gutter.height,
            colors.gutter_separator,
        );

        // Status bar background
        geometry.add_rect(
            layout,
            layout.status_bar.x,
            layout.status_bar.y,
            layout.status_bar.width,
            layout.status_bar.height,
            colors.status_bar_background,
        );

        // Status bar top separator
        geometry.add_rect(
            layout,
            layout.status_bar.x,
            layout.status_bar.y,
            layout.status_bar.width,
            1.0,
            colors.gutter_separator,
        );

        geometry
    }

    fn add_rect(
        &mut self,
        layout: &EditorLayout,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [f32; 4],
    ) {
        let [x1, y1] = layout.pixel_to_ndc(x, y);
        let [x2, y2] = layout.pixel_to_ndc(x + width, y + height);

        let vertex_start = self.vertices.len() as u32;

        // Top-left
        self.vertices.push(ColorVertex {
            position: [x1, y1],
            color,
        });

        // Top-right
        self.vertices.push(ColorVertex {
            position: [x2, y1],
            color,
        });

        // Bottom-right
        self.vertices.push(ColorVertex {
            position: [x2, y2],
            color,
        });

        // Bottom-left
        self.vertices.push(ColorVertex {
            position: [x1, y2],
            color,
        });

        // Two triangles
        self.indices.push(vertex_start);
        self.indices.push(vertex_start + 1);
        self.indices.push(vertex_start + 2);

        self.indices.push(vertex_start);
        self.indices.push(vertex_start + 2);
        self.indices.push(vertex_start + 3);
    }
}
