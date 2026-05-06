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

    /// Return the `\definecolor` declaration for this color with the given name.
    pub fn define(&self, name: &str) -> String {
        format!("\\definecolor{{{name}}}{{RGB}}{{{},{},{}}}", self.r, self.g, self.b)
    }
}

/// Named palette entries expected to be defined in the LaTeX preamble.
///
/// The generated TikZ uses these identifiers directly; no RGB values are
/// embedded in generated plot output.
pub mod palette {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ColorName {
        Energy,
        EnergyComplementary,
        Runtime,
        RuntimeComplementary,
        Colorblind(usize),
        Named(&'static str),
    }

    /// Primary energy / efficiency bar color.
    pub const ENERGY: ColorName = ColorName::Energy;
    /// Complementary (lighter) energy color for IQR bands.
    pub const ENERGY_COMPLEMENTARY: ColorName = ColorName::EnergyComplementary;
    /// Primary runtime / throughput line color.
    pub const RUNTIME: ColorName = ColorName::Runtime;
    /// Complementary (lighter) runtime color for IQR bands.
    pub const RUNTIME_COMPLEMENTARY: ColorName = ColorName::RuntimeComplementary;

    // Backward-compatible aliases used by public examples/tests.
    pub const GREEN: ColorName = ENERGY;
    pub const LIGHT_GREEN: ColorName = ENERGY_COMPLEMENTARY;
    pub const RED: ColorName = RUNTIME;
    pub const BLUE: ColorName = ColorName::Colorblind(0);
    pub const ORANGE: ColorName = ColorName::Colorblind(1);
    pub const PURPLE: ColorName = ColorName::Colorblind(4);
    pub const GREY: ColorName = ColorName::Colorblind(7);
    pub const BLACK: ColorName = ColorName::Named("black");
    pub const WHITE: ColorName = ColorName::Named("white");

    /// Seaborn colorblind palette identifiers (`epycolorblind0`…`epycolorblind9`).
    pub const COLORBLIND: [&str; 10] = [
        "epycolorblind0",
        "epycolorblind1",
        "epycolorblind2",
        "epycolorblind3",
        "epycolorblind4",
        "epycolorblind5",
        "epycolorblind6",
        "epycolorblind7",
        "epycolorblind8",
        "epycolorblind9",
    ];

    pub fn colorblind(index: usize) -> &'static str {
        COLORBLIND[index % COLORBLIND.len()]
    }

    impl ColorName {
        pub fn tikz_name(self) -> &'static str {
            match self {
                ColorName::Energy => "epyenergycolor",
                ColorName::EnergyComplementary => "epyenergycomplementary",
                ColorName::Runtime => "epyruntimecolor",
                ColorName::RuntimeComplementary => "epyruntimecomplementary",
                ColorName::Colorblind(idx) => colorblind(idx),
                ColorName::Named(name) => name,
            }
        }

        pub fn complementary(self) -> Self {
            match self {
                ColorName::Energy => ColorName::EnergyComplementary,
                ColorName::Runtime => ColorName::RuntimeComplementary,
                _ => self,
            }
        }
    }
}
