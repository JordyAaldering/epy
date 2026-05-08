mod color;
mod data;
mod ir;
mod plot;

pub mod prelude {
    pub use crate::color::*;
    pub use crate::data::*;
    pub use crate::plot::*;
}

use prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct Record {
    #[serde(skip)]
    _i_also_use_name_columns_sometimes: String,
    threads: usize,
    #[serde(deserialize_with = "from_micro")]
    powercap: f64,
    insns: usize,
    rapl: f64,
    runtime: f64,
    cycs: usize,
}

fn from_micro<'de, D>(de: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let x = usize::deserialize(de)?;
    Ok(x as f64 / 1e6)
}

impl Record {
    fn gflop_s(&self) -> f64 {
        self.insns as f64 / self.runtime / 1e9
    }

    fn gflop_j(&self) -> f64 {
        self.insns as f64 / self.rapl / 1e9
    }

    fn ipc(&self) -> f64 {
        self.insns as f64 / self.cycs as f64
    }
}

/// Return the maximum thread count present in the data.
fn max_threads(df: &DataFrame<Record>) -> usize {
    df.rows()
        .iter()
        .map(|r| r.threads)
        .max()
        .unwrap()
}

fn twin(df: &DataFrame<Record>) {
    let max_t = max_threads(df);
    let filtered = df.clone().filter(|r| r.threads == max_t);

    let mut doc = TwinPlot::new(
            filtered,
            |r| r.powercap,
            |r| r.gflop_j(), "GFLOP/J",
            |r| r.gflop_s(), "GFLOP/s",
            "Power limit (W)",
        )
        .build_document();

    // Post-build modifications
    doc.ax0.set_ymax(0.09);
    doc.ax0.set_legend_pos("south east");
    doc.ax1.as_mut().unwrap().set_ymax(0.6);
    doc.ax0.filter_xticks_stride(2);
    doc.ax0.format_xticks_precision(1);

    let tikz = doc.render_tikz();

    std::fs::write(".build/example_twin.tex", tikz).unwrap();
}

fn ipc(df: &DataFrame<Record>) {
    let max_t = max_threads(df);
    let filtered = df.clone().filter(|r| r.threads == max_t);

    let tikz = LinePlot::new(filtered, |r| r.powercap, "Power limit (W)", "IPC")
        .series(|r| r.ipc(), "IPC", Color::Runtime)
        .build_document()
        .annot_area((3.0 + 3.5) / 2.0, 0.0, (7.125 + 7.75) / 2.0, 1.0, Color::Annot)
        .annot_label(7.0, 0.25, "Hello, world!")
        .render_tikz();

    std::fs::write(".build/example_ipc.tex", tikz).unwrap();
}

fn zplot(df: &DataFrame<Record>) {
    let tikz = ZPlot::new(
            df.clone(),
            |r| r.threads as f64,
            |r| r.powercap,
            |r| r.gflop_s(),
            |r| r.gflop_j(),
            "GFLOP/s",
            "GFLOP/J",
        )
        .build_document()
        .render_tikz();

    std::fs::write(".build/example_zplot.tex", tikz).unwrap();
}

fn main() {
    std::fs::create_dir_all(".build").unwrap();
    let df = DataFrame::from_csv("test_data.csv").unwrap();
    twin(&df);
    ipc(&df);
    zplot(&df);
}
