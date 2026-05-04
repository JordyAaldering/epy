//! Color palette for energy-measurement plots.

/// An RGB color for use in generated TikZ output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Construct a color from RGB components.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    /// Construct a color from a 6-digit hex string like `"#3AA640"`.
    ///
    /// # Panics
    /// Panics if the string is not a valid hex color.
    pub fn hex(s: &str) -> Self {
        let s = s.trim_start_matches('#');
        assert!(s.len() == 6, "expected 6-digit hex color, got `#{s}`");
        let r = u8::from_str_radix(&s[0..2], 16).expect("invalid red component");
        let g = u8::from_str_radix(&s[2..4], 16).expect("invalid green component");
        let b = u8::from_str_radix(&s[4..6], 16).expect("invalid blue component");
        Color { r, g, b }
    }

    /// Return the `\definecolor` declaration for this color with the given name.
    pub fn define(&self, name: &str) -> String {
        format!("\\definecolor{{{name}}}{{RGB}}{{{},{},{}}}", self.r, self.g, self.b)
    }
}

/// Predefined palette matching the project's Python color constants.
pub mod palette {
    use super::Color;

    /// Primary energy / bar-chart color.
    pub const GREEN: Color = Color::rgb(58, 166, 64);
    /// Secondary / lighter green.
    pub const LIGHT_GREEN: Color = Color::rgb(168, 212, 122);
    /// Runtime / line-chart color.
    pub const RED: Color = Color::rgb(166, 58, 58);
    /// IPC / derived-metric color.
    pub const BLUE: Color = Color::rgb(58, 106, 166);
    pub const ORANGE: Color = Color::rgb(166, 118, 58);
    pub const PURPLE: Color = Color::rgb(160, 58, 166);
    pub const GREY: Color = Color::rgb(170, 171, 171);
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);

    /// Ordered palette for multi-series plots (e.g. Z-plots with several thread counts).
    pub const SERIES: [Color; 8] = [GREEN, RED, BLUE, ORANGE, PURPLE, LIGHT_GREEN, GREY, BLACK];
}
