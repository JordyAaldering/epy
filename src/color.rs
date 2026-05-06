#[derive(Copy, Clone, Debug)]
pub enum Color {
    Energy,
    EnergyComplementary,
    Runtime,
    RuntimeComplementary,
    Colorblind(usize),
}

impl Color {
    pub fn tikz_name(self) -> String {
        use Color::*;
        match self {
            Energy => "epyenergycolor".to_owned(),
            EnergyComplementary => "epyenergycomplementary".to_owned(),
            Runtime => "epyruntimecolor".to_owned(),
            RuntimeComplementary => "epyruntimecomplementary".to_owned(),
            Colorblind(idx) => format!("epycolorblind{}", idx),
        }
    }

    pub fn complementary(self) -> Self {
        use Color::*;
        match self {
            Energy => EnergyComplementary,
            Runtime => RuntimeComplementary,
            _ => panic!("complementary color not defined for {:?}", self),
        }
    }
}
