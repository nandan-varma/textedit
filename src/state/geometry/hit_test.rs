use super::super::init::State;

impl State {
    pub fn window(&self) -> &std::sync::Arc<winit::window::Window> {
        &self.window
    }

    pub fn get_char_at_position(
        &self,
        x: f64,
        y: f64,
        buffer: &crate::domain::Buffer,
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
}
