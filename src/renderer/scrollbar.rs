use super::layout::EditorLayout;
use super::ui_background::ColorVertex;

pub struct ScrollbarGeometry {
    pub vertices: Vec<ColorVertex>,
    pub indices: Vec<u32>,
}

impl ScrollbarGeometry {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Build geometry for a simple vertical scrollbar.
    ///
    /// `total_visual_lines` is the total number of wrapped visual lines in the buffer.
    /// `visible_lines` is how many visual lines fit in the viewport.
    /// `scroll_offset` is the index of the first visual line currently visible.
    pub fn build(
        layout: &EditorLayout,
        colors: &super::layout::Colors,
        total_visual_lines: usize,
        visible_lines: usize,
        scroll_offset: usize,
    ) -> Self {
        let mut geom = ScrollbarGeometry::new();

        if total_visual_lines == 0 || visible_lines == 0 || total_visual_lines <= visible_lines {
            // No scrollbar needed if everything fits.
            return geom;
        }

        let track = layout.scrollbar_area;
        let [track_x1, track_y1] = layout.pixel_to_ndc(track.x, track.y);
        let [track_x2, track_y2] =
            layout.pixel_to_ndc(track.x + track.width, track.y + track.height);

        let track_color = colors.scrollbar_track;

        let track_vertex_start = geom.vertices.len() as u32;

        // Track quad
        geom.vertices.push(ColorVertex {
            position: [track_x1, track_y1],
            color: track_color,
        });
        geom.vertices.push(ColorVertex {
            position: [track_x2, track_y1],
            color: track_color,
        });
        geom.vertices.push(ColorVertex {
            position: [track_x2, track_y2],
            color: track_color,
        });
        geom.vertices.push(ColorVertex {
            position: [track_x1, track_y2],
            color: track_color,
        });

        geom.indices.push(track_vertex_start);
        geom.indices.push(track_vertex_start + 1);
        geom.indices.push(track_vertex_start + 2);
        geom.indices.push(track_vertex_start);
        geom.indices.push(track_vertex_start + 2);
        geom.indices.push(track_vertex_start + 3);

        // Thumb
        let max_offset = total_visual_lines.saturating_sub(visible_lines).max(1);
        let ratio = (scroll_offset.min(max_offset) as f32) / (max_offset as f32);

        let thumb_min_height = layout.line_height.max(8.0);
        let thumb_height =
            (track.height * (visible_lines as f32 / total_visual_lines as f32)).max(thumb_min_height);

        let travel = (track.height - thumb_height).max(0.0);
        let thumb_top = track.y + ratio * travel;

        let [thumb_x1, thumb_y1] = layout.pixel_to_ndc(track.x, thumb_top);
        let [thumb_x2, thumb_y2] =
            layout.pixel_to_ndc(track.x + track.width, thumb_top + thumb_height);

        let thumb_color = colors.scrollbar_thumb;
        let thumb_vertex_start = geom.vertices.len() as u32;

        geom.vertices.push(ColorVertex {
            position: [thumb_x1, thumb_y1],
            color: thumb_color,
        });
        geom.vertices.push(ColorVertex {
            position: [thumb_x2, thumb_y1],
            color: thumb_color,
        });
        geom.vertices.push(ColorVertex {
            position: [thumb_x2, thumb_y2],
            color: thumb_color,
        });
        geom.vertices.push(ColorVertex {
            position: [thumb_x1, thumb_y2],
            color: thumb_color,
        });

        geom.indices.push(thumb_vertex_start);
        geom.indices.push(thumb_vertex_start + 1);
        geom.indices.push(thumb_vertex_start + 2);
        geom.indices.push(thumb_vertex_start);
        geom.indices.push(thumb_vertex_start + 2);
        geom.indices.push(thumb_vertex_start + 3);

        geom
    }
}

