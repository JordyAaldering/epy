mod data;
mod ir;
mod plot;

pub mod prelude {
    pub use crate::data::*;
    pub use crate::ir::*;
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

    let (mut ax0, mut ax1) = TwinPlot::new(
            filtered,
            |r| r.powercap,
            |r| r.gflop_j(), "GFLOP/J",
            |r| r.gflop_s(), "GFLOP/s",
            "Power limit (W)",
        )
        .build_axes();

    ax0 = ax0.set_ymax(0.09)
        .set_legend_pos("south east")
        .filter_xticks_stride(0, 2)
        .format_xticks_precision(1, false);
    ax1 = ax1.set_ymax(0.6);

    let doc = ir::PlotDocument::from_twin_axes(ax0, ax1);

    let tikz = doc.render_tikz();

    std::fs::write(".build/example_twin.tex", tikz).unwrap();
}

fn ipc(df: &DataFrame<Record>) {
    let max_t = max_threads(df);
    let filtered = df.clone().filter(|r| r.threads == max_t);

    let ax0 = LinePlot::new(filtered, |r| r.powercap, "Power limit (W)", "IPC")
        .series(|r| r.ipc(), "IPC", "runtimecolor".into())
        .build_axis()
        .filter_xticks_stride(0, 2)
        .format_xticks_precision(1, false);

    let doc = ir::PlotDocument::from_axis(ax0)
        .annot_area((5.5, 0.0), (12.5, 1.0), "black!30".into())
        .annot_label((5.0, 0.35), "Hello, world!");

    let tikz = doc.render_tikz();

    std::fs::write(".build/example_ipc.tex", tikz).unwrap();
}

fn power(df: &DataFrame<Record>) {
    let grouped = df.clone().group_by(|r| r.threads as f64);

    let ax = LineGrouped::new(grouped,
        |r| r.powercap,
        |r| r.rapl / r.runtime,
        "Configured power limit (W)",
        "Actual power draw (W)")
        .build_axis();

    let doc = ir::PlotDocument::from_axis(ax);
    let tikz = doc.render_tikz();
    std::fs::write(".build/example_power.tex", tikz).unwrap();
}

fn zplot(df: &DataFrame<Record>) {
    let ax = ZPlot::new(
            df.clone(),
            |r| r.threads as f64,
            |r| r.powercap,
            |r| r.gflop_s(),
            |r| r.gflop_j(),
            "GFLOP/s",
            "GFLOP/J",
        )
        .build_axis()
        .set_xmin(0.0)
        .set_ymin(0.0);

    let doc = ir::PlotDocument::from_axis(ax);

    let tikz = doc.render_tikz();

    std::fs::write(".build/example_zplot.tex", tikz).unwrap();
}

fn main() {
    std::fs::create_dir_all(".build").unwrap();
    let df = DataFrame::from_csv("test_data.csv").unwrap();
    twin(&df);
    ipc(&df);
    power(&df);
    zplot(&df);
}
