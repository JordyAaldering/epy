mod color;
mod data;
mod plot;
mod ir;
mod stats;

pub mod prelude {
    pub use crate::color::*;
    pub use crate::data::*;
    pub use crate::plot::*;
    pub use crate::stats::*;
}

use serde::Deserialize;

use prelude::*;

#[derive(Deserialize)]
struct Record {
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

fn twin() {
    let df = DataFrame::<Record>::from_csv("test_data.csv").unwrap();
    let example_threads = df
        .rows()
        .last()
        .expect("example fixture must contain at least one row");
    let example_threads = example_threads.threads;

    let grouped = df
        .filter(|r| r.threads == example_threads)
        .group_by(|r| r.powercap);

    let tikz = TwinPlot::new(
            grouped,
            |r| r.gflop_j(),
            "GFLOP/J",
            |r| r.gflop_s(),
            "GFLOP/s",
            "Power limit (W)",
        )
        .build_document()
        .render_tikz();

    std::fs::write(".build/example_twin.tex", tikz).unwrap();
}

fn ipc() {
    let df = DataFrame::<Record>::from_csv("test_data.csv").unwrap();
    let example_threads = df
        .rows()
        .last()
        .expect("example fixture must contain at least one row");
    let example_threads = example_threads.threads;

    let grouped = df
        .filter(|r| r.threads == example_threads)
        .group_by(|r| r.powercap);

    let tikz = LinePlot::new(
            grouped,
            "Power limit (W)",
            "IPC",
        )
        .series(|r| r.ipc(), "IPC", Color::Runtime)
        .build_document()
        .annot_area((3.0 + 3.5) / 2.0, 0.0, (7.125 + 7.75) / 2.0, 1.0, Color::Annot)
        .annot_label(7.0, 0.25, "Hello, world!")
        .render_tikz();

    std::fs::write(".build/example_ipc.tex", tikz).unwrap();
}

fn zplot() {
    let grouped = DataFrame::<Record>::from_csv("test_data.csv")
        .unwrap()
        .group_by(|r| r.threads as f64);

    let tikz = ZPlot::new(
            grouped,
            |r| r.gflop_s(),
            |r| r.gflop_j(),
            |r| r.powercap as f64,
            "GFLOP/s",
            "GFLOP/J",
        )
        .build_document()
        .render_tikz();

    std::fs::write(".build/example_zplot.tex", tikz).unwrap();
}

fn main() {
    twin();
    ipc();
    zplot();
}
