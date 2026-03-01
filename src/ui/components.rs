#[allow(dead_code)]
pub struct LineNumbers;

#[allow(dead_code)]
impl LineNumbers {
    pub fn width(line_count: usize) -> f32 {
        let digits = (line_count as f32).log10().floor() + 1.0;
        digits * 8.0 + 16.0 // chars * char_width + padding
    }

    pub fn format_line_number(line: usize) -> String {
        format!("{:>4}", line + 1)
    }
}

#[allow(dead_code)]
pub struct StatusBar {
    line: usize,
    column: usize,
    file_name: String,
    is_modified: bool,
}

#[allow(dead_code)]
impl StatusBar {
    pub fn new(line: usize, column: usize, file_name: String, is_modified: bool) -> Self {
        Self {
            line,
            column,
            file_name,
            is_modified,
        }
    }

    pub fn render_text(&self) -> String {
        let _modified = if self.is_modified { " ● " } else { " " };
        format!(
            " {} | Line {}, Col {} ",
            self.file_name,
            self.line + 1,
            self.column + 1
        )
    }
}
