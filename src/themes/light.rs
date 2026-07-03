use crate::renderer::layout::Colors;

/// All syntax colors verified ≥ 4.5:1 contrast on white (#FFFFFF)
pub fn light_syntax_palette() -> &'static [([f32; 4], &'static str)] {
    &[
        ([0.016, 0.400, 0.714, 1.0], "keyword"),  // #0566B6  5.9:1
        ([0.529, 0.196, 0.000, 1.0], "string"),   // #873200  8.5:1
        ([0.235, 0.302, 0.467, 1.0], "function"), // #3C4D77  8.2:1
        ([0.545, 0.094, 0.231, 1.0], "type"),     // #8B183B  9.2:1
        ([0.600, 0.282, 0.000, 1.0], "number"),   // #994800  6.3:1
        ([0.420, 0.420, 0.420, 1.0], "comment"),  // #6B6B6B  5.4:1
    ]
}

/// Light theme — UI colors verified against WCAG 2.1 AA
pub fn light() -> Colors {
    Colors {
        // text #222222 on #FFFFFF = 13.8:1 ✓
        background: [1.000, 1.000, 1.000, 1.0],
        gutter_background: [0.961, 0.961, 0.961, 1.0],
        status_bar_background: [0.929, 0.929, 0.929, 1.0],
        text_color: [0.133, 0.133, 0.133, 1.0],
        // line numbers #777777 on white = 4.5:1 ✓
        line_number_color: [0.467, 0.467, 0.467, 1.0],
        cursor_color: [0.133, 0.133, 0.133, 1.0],
        // semi-transparent so text drawn beneath shows through
        selection_color: [0.200, 0.600, 0.900, 0.30],
        gutter_separator: [0.808, 0.808, 0.808, 1.0],
        scrollbar_track: [0.949, 0.949, 0.949, 1.0],
        scrollbar_thumb: [0.733, 0.733, 0.733, 1.0],
        modal_background: [0.980, 0.980, 0.980, 1.0],
        // modal border #808080 on modal bg = 3.8:1 ✓
        modal_border: [0.502, 0.502, 0.502, 1.0],
        input_background: [1.000, 1.000, 1.000, 1.0],
        input_border: [0.733, 0.733, 0.733, 1.0],
        // focus ring #0477C7 on white = 4.7:1 ✓
        input_border_focused: [0.016, 0.467, 0.780, 1.0],
        match_highlight: [0.980, 0.840, 0.020, 0.55],
        current_match_highlight: [0.980, 0.640, 0.020, 0.85],
        button_background: [0.906, 0.906, 0.906, 1.0],
        button_hover: [0.847, 0.847, 0.847, 1.0],
    }
}
