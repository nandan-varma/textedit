use super::super::init::State;
use crate::ui::modal::ModalState;
use wgpu::util::DeviceExt;

impl State {
    pub fn update_geometry(
        &mut self,
        buffer: &crate::domain::Buffer,
        cursor: &crate::domain::Cursor,
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
        let colors = &crate::renderer::layout::Colors::default();
        let ui_bg = crate::renderer::ui_background::UIBackgroundGeometry::build(&layout, colors);
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
            let wrapped_text = crate::renderer::text_geometry::WrappedText::wrap_buffer(
                buffer,
                glyph_atlas,
                &layout,
            );

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
                colors,
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
                colors,
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
            let status_bar = crate::renderer::status_bar::StatusBarGeometry::build(
                cursor,
                buffer,
                glyph_atlas,
                &layout,
                colors,
                status_bar_override,
            )
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
                        bytes_per_row: Some(crate::state::GLYPH_ATLAS_SIZE),
                        rows_per_image: Some(crate::state::GLYPH_ATLAS_SIZE),
                    },
                    wgpu::Extent3d {
                        width: crate::state::GLYPH_ATLAS_SIZE,
                        height: crate::state::GLYPH_ATLAS_SIZE,
                        depth_or_array_layers: 1,
                    },
                );
            }

            // Update cursor, selection, and scrollbar geometry using current scrolling state
            let cursor_geom = crate::renderer::cursor::CursorGeometry::build_with_wrap(
                cursor,
                buffer,
                &layout,
                glyph_atlas,
                self.scroll_visual_offset,
                colors,
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
                colors,
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

    /// Update modal geometry (called when modal state changes)
    pub fn update_modal_geometry(
        &mut self,
        modal_state: &ModalState,
        matches: &[(usize, usize)],
        current_match: Option<usize>,
        buffer: &crate::domain::Buffer,
        show_line_numbers: bool,
        show_status_bar: bool,
        cursor_visible: bool,
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
        let colors = self.config.colors();

        // Update match highlights
        if let Some(glyph_atlas) = &mut self.glyph_atlas {
            let match_geometry = crate::renderer::cursor::CursorGeometry::build_match_highlights(
                buffer,
                matches,
                current_match,
                &layout,
                glyph_atlas,
                self.scroll_visual_offset,
                &colors,
            );

            if !match_geometry.vertices.is_empty() {
                self.match_highlight_vertex_buffer = Some(self.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("match highlight vertex buffer"),
                        contents: bytemuck::cast_slice(&match_geometry.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                ));
                self.match_highlight_index_buffer = Some(self.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("match highlight index buffer"),
                        contents: bytemuck::cast_slice(&match_geometry.indices),
                        usage: wgpu::BufferUsages::INDEX,
                    },
                ));
                self.match_highlight_index_count = match_geometry.indices.len() as u32;
            } else {
                self.match_highlight_index_count = 0;
            }

            // Update modal geometry if modal is open
            match modal_state {
                ModalState::None => {
                    self.modal_bg_index_count = 0;
                    self.modal_text_index_count = 0;
                    // Clear hit test regions when modal is closed
                    self.modal_button_regions.clear();
                    self.modal_input_regions.clear();
                    self.modal_rect = None;
                }
                ModalState::Find(find_modal) => {
                    let modal_geometry = crate::renderer::modal::FindModalGeometry::build(
                        find_modal,
                        &layout,
                        glyph_atlas,
                        &colors,
                        cursor_visible,
                    );

                    if !modal_geometry.bg_vertices.is_empty() {
                        self.modal_bg_vertex_buffer = Some(self.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("modal bg vertex buffer"),
                                contents: bytemuck::cast_slice(&modal_geometry.bg_vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            },
                        ));
                        self.modal_bg_index_buffer = Some(self.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("modal bg index buffer"),
                                contents: bytemuck::cast_slice(&modal_geometry.bg_indices),
                                usage: wgpu::BufferUsages::INDEX,
                            },
                        ));
                        self.modal_bg_index_count = modal_geometry.bg_indices.len() as u32;
                    }

                    if !modal_geometry.text_vertices.is_empty() {
                        self.modal_text_vertex_buffer = Some(self.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("modal text vertex buffer"),
                                contents: bytemuck::cast_slice(&modal_geometry.text_vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            },
                        ));
                        self.modal_text_index_buffer = Some(self.device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some("modal text index buffer"),
                                contents: bytemuck::cast_slice(&modal_geometry.text_indices),
                                usage: wgpu::BufferUsages::INDEX,
                            },
                        ));
                        self.modal_text_index_count = modal_geometry.text_indices.len() as u32;
                    }

                    // Store hit test regions for mouse click handling
                    self.modal_button_regions = modal_geometry.button_regions;
                    self.modal_input_regions = modal_geometry.input_regions;
                    self.modal_rect = Some(modal_geometry.modal_rect);

                    // Update atlas texture after modal rendering
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
                                bytes_per_row: Some(crate::state::GLYPH_ATLAS_SIZE),
                                rows_per_image: Some(crate::state::GLYPH_ATLAS_SIZE),
                            },
                            wgpu::Extent3d {
                                width: crate::state::GLYPH_ATLAS_SIZE,
                                height: crate::state::GLYPH_ATLAS_SIZE,
                                depth_or_array_layers: 1,
                            },
                        );
                    }
                }
            }
        }

        Ok(())
    }
}
