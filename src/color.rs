#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    /// Color used for grid lines and axis lines.
    Grid,
    /// Color used for annotations, like highlighted areas.
    Annot,
    /// Color used exclusively for energy-related data.
    Energy,
    /// Color used exclusively for runtime-related data.
    Runtime,
    /// Colorblind-friendly palette for categorical data. The usize is an index into the palette.
    Colorblind(usize),
}

impl Color {
    pub fn tikz_name(self) -> String {
        use Color::*;
        match self {
            Grid => "black!20".into(),
            Annot => "black!30".into(),
            Energy => "epyenergycolor".into(),
            Runtime => "epyruntimecolor".into(),
            Colorblind(idx) => format!("epycolorblind{}", idx),
        }
    }
}
