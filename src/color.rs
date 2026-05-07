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
    /// Complementary energy color, useful when a plot contains a secondary series.
    EnergyComplementary,
    /// Complementary runtime color, useful when a plot contains a secondary series.
    RuntimeComplementary,
    /// Colorblind-friendly palette for categorical data. The usize is an index into the palette.
    Colorblind(usize),
}

impl Color {
    pub fn tikz_name(self) -> String {
        use Color::*;
        match self {
            Grid => "epygridcolor".to_owned(),
            Annot => "epyannotcolor".to_owned(),
            Energy => "epyenergycolor".to_owned(),
            Runtime => "epyruntimecolor".to_owned(),
            EnergyComplementary => "epyenergycompl".to_owned(),
            RuntimeComplementary => "epyruntimecompl".to_owned(),
            Colorblind(idx) => format!("epycolorblind{}", idx),
        }
    }
}
