mod color;
mod data;
mod plot;
mod ir;
mod stats;

// Need this here to avoid unused warnings
pub mod prelude {
    pub use crate::color::Color;
    pub use crate::data::{DataFrame, GroupedFrame};
    pub use crate::plot::{TwinPlot, LinePlot, ZPlot};
    pub use crate::stats::{mean, median, q1, q3};
}

use prelude::*;

fn twin() {
    let df = DataFrame::from_csv("test_data.csv").unwrap();
    let example_threads = *df
        .col("threads")
        .last()
        .expect("example fixture must contain at least one row");

    let df = df
        .filter(|r| r["threads"] == example_threads)
        .with_column("powercapW", |r| r["powercap"] / 1e6)
        .with_column("gflop_j", |r| r["insns"] / r["rapl"] / 1e9)
        .with_column("gflop_s", |r| r["insns"] / r["runtime"] / 1e9);

    let grouped = df.group_by("powercapW");

    let tikz = TwinPlot::new(
            grouped,
            "gflop_j",
            "GFLOP/J",
            "gflop_s",
            "GFLOP/s",
            "Power limit (W)",
        )
        .build_document()
        .render_tikz();

    std::fs::write(".build/example_twin.tex", tikz).unwrap();
}

fn ipc() {
    let df = DataFrame::from_csv("test_data.csv").unwrap();
    let example_threads = *df
        .col("threads")
        .last()
        .expect("example fixture must contain at least one row");

    let df = df
        .filter(|r| r["threads"] == example_threads)
        .with_column("powercapW", |r| r["powercap"] / 1e6)
        .with_column("ipc", |r| r["insns"] / r["cycs"]);

    let grouped = df.group_by("powercapW");

    let tikz = LinePlot::new(
            grouped,
            "Power limit (W)",
            "IPC",
        )
        .series("ipc", "IPC", Color::Runtime)
        .build_document()
        .render_tikz();

    std::fs::write(".build/example_ipc.tex", tikz).unwrap();
}

fn main() {
    twin();
    ipc();
}
