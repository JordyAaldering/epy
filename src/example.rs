pub mod color;
pub mod data;
pub mod plot;
pub mod stats;

use crate::{color::*, data::DataFrame, plot::BarLinePlot};

fn main() {
    let df = DataFrame::from_csv("tests/fixtures/test_data.csv").unwrap()
        .filter(|r| r["threads"] == 8.0)
        .with_column("gflop_j", |r| r["insns"] / r["rapl"] / 1e9)
        .with_column("gflop_s", |r| r["insns"] / r["runtime"] / 1e9);

    let grouped = df.group_by("powercap");

    // Twin-axis bar (efficiency) + line (throughput) plot
    let tikz = BarLinePlot::new(grouped)
        .bar("gflop_j", palette::GREEN, r"GFLOP/J")
        .line("gflop_s", palette::RED,  r"GFLOP/s")
        .xlabel(r"Power limit (W)")
        .render();

    std::fs::write("example_tikz.tex", tikz).unwrap();
}
