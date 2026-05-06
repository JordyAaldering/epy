#[derive(Clone, Copy, Debug)]
pub enum Color {
    Energy,
    Runtime,
    EnergyComplementary,
    RuntimeComplementary,
    Colorblind(usize),
}

impl Color {
    pub fn tikz_name(self) -> String {
        use Color::*;
        match self {
            Energy => "epyenergycolor".to_owned(),
            Runtime => "epyruntimecolor".to_owned(),
            EnergyComplementary => "epyenergycompl".to_owned(),
            RuntimeComplementary => "epyruntimecompl".to_owned(),
            Colorblind(idx) => format!("epycolorblind{}", idx),
        }
    }
}
