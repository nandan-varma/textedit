/// UI Layout constants and calculations for the text editor
/// All measurements are in base pixels (before scaling)

// Base sizes (at 1x scale)
pub const BASE_LINE_NUMBER_GUTTER_WIDTH: f32 = 50.0;
pub const BASE_LINE_NUMBER_PADDING_RIGHT: f32 = 10.0;
pub const BASE_TEXT_AREA_PADDING_LEFT: f32 = 10.0;
pub const BASE_TEXT_AREA_PADDING_TOP: f32 = 5.0;
pub const BASE_TEXT_AREA_PADDING_RIGHT: f32 = 10.0;
pub const BASE_STATUS_BAR_HEIGHT: f32 = 24.0;
pub const BASE_STATUS_BAR_PADDING: f32 = 8.0;

/// Colors for the UI (RGBA, 0.0-1.0)
pub struct Colors;

impl Colors {
    pub const BACKGROUND: [f32; 4] = [0.12, 0.12, 0.12, 1.0]; // Dark gray background
    pub const GUTTER_BACKGROUND: [f32; 4] = [0.15, 0.15, 0.15, 1.0]; // Slightly lighter gutter
    pub const STATUS_BAR_BACKGROUND: [f32; 4] = [0.18, 0.18, 0.18, 1.0]; // Status bar bg
    pub const TEXT_COLOR: [f32; 4] = [0.92, 0.92, 0.92, 1.0]; // Light text
    pub const LINE_NUMBER_COLOR: [f32; 4] = [0.5, 0.5, 0.5, 1.0]; // Dimmer line numbers
    pub const CURSOR_COLOR: [f32; 4] = [0.9, 0.9, 0.9, 1.0]; // Bright cursor
    pub const SELECTION_COLOR: [f32; 4] = [0.3, 0.5, 0.7, 0.7]; // Selection highlight (blue-ish)
    pub const GUTTER_SEPARATOR: [f32; 4] = [0.25, 0.25, 0.25, 1.0]; // Separator line
}

/// Represents a rectangular area in pixel coordinates (top-left origin)
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }
}

/// Complete layout of the editor UI
pub struct EditorLayout {
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub gutter: Rect,
    pub text_area: Rect,
    pub status_bar: Rect,
    pub font_size: f32,
    pub line_height: f32,
    pub char_width: f32,
    pub scale_factor: f32,
    // Scaled constants
    pub gutter_width: f32,
    pub line_number_padding_right: f32,
    pub text_area_padding_left: f32,
    pub text_area_padding_top: f32,
    pub text_area_padding_right: f32,
    pub status_bar_height: f32,
    pub status_bar_padding: f32,
    // Show flags
    pub show_line_numbers: bool,
    pub show_status_bar: bool,
}

impl EditorLayout {
    pub fn new(
        viewport_width: f32,
        viewport_height: f32,
        font_size: f32,
        scale_factor: f32,
        show_line_numbers: bool,
        show_status_bar: bool,
    ) -> Self {
        let line_height = (font_size * 1.4).round();
        let char_width = (font_size * 0.6).round();

        // Scale all layout constants
        let gutter_width = if show_line_numbers {
            (BASE_LINE_NUMBER_GUTTER_WIDTH * scale_factor).round()
        } else {
            0.0
        };
        let line_number_padding_right = (BASE_LINE_NUMBER_PADDING_RIGHT * scale_factor).round();
        let text_area_padding_left = (BASE_TEXT_AREA_PADDING_LEFT * scale_factor).round();
        let text_area_padding_top = (BASE_TEXT_AREA_PADDING_TOP * scale_factor).round();
        let text_area_padding_right = (BASE_TEXT_AREA_PADDING_RIGHT * scale_factor).round();
        let status_bar_height = if show_status_bar {
            (BASE_STATUS_BAR_HEIGHT * scale_factor).round()
        } else {
            0.0
        };
        let status_bar_padding = (BASE_STATUS_BAR_PADDING * scale_factor).round();

        let gutter = Rect::new(0.0, 0.0, gutter_width, viewport_height - status_bar_height);

        let text_area = Rect::new(
            gutter_width,
            0.0,
            viewport_width - gutter_width,
            viewport_height - status_bar_height,
        );

        let status_bar = Rect::new(
            0.0,
            viewport_height - status_bar_height,
            viewport_width,
            status_bar_height,
        );

        Self {
            viewport_width,
            viewport_height,
            gutter,
            text_area,
            status_bar,
            font_size,
            line_height,
            char_width,
            scale_factor,
            gutter_width,
            line_number_padding_right,
            text_area_padding_left,
            text_area_padding_top,
            text_area_padding_right,
            status_bar_height,
            status_bar_padding,
            show_line_numbers,
            show_status_bar,
        }
    }

    /// Convert pixel coordinates to NDC (-1 to 1)
    /// Pixel origin is top-left, NDC origin is center
    pub fn pixel_to_ndc(&self, x: f32, y: f32) -> [f32; 2] {
        let ndc_x = (x / self.viewport_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (y / self.viewport_height) * 2.0;
        [ndc_x, ndc_y]
    }

    /// Convert pixel width/height to NDC scale
    pub fn size_to_ndc(&self, width: f32, height: f32) -> [f32; 2] {
        let ndc_w = (width / self.viewport_width) * 2.0;
        let ndc_h = (height / self.viewport_height) * 2.0;
        [ndc_w, ndc_h]
    }

    /// Get the pixel position for a text character at given line and column
    pub fn text_position(&self, line: usize, column: usize) -> [f32; 2] {
        let x = self.text_area.x + self.text_area_padding_left + (column as f32 * self.char_width);
        let y = self.text_area_padding_top + (line as f32 * self.line_height);
        [x, y]
    }

    /// Get the pixel position for a line number
    pub fn line_number_position(&self, line: usize, num_digits: usize) -> [f32; 2] {
        // Right-align line numbers in gutter
        let text_width = num_digits as f32 * self.char_width;
        let x = self.gutter.width - self.line_number_padding_right - text_width;
        let y = self.text_area_padding_top + (line as f32 * self.line_height);
        [x, y]
    }

    /// Get cursor position in pixels for given line and column
    pub fn cursor_position(&self, line: usize, column: usize) -> [f32; 2] {
        self.text_position(line, column)
    }

    /// Number of visible lines in the text area
    pub fn visible_lines(&self) -> usize {
        ((self.text_area.height - self.text_area_padding_top * 2.0) / self.line_height).floor()
            as usize
    }
}
