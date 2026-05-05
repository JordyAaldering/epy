pub mod color;
pub mod data;
pub mod plot;
pub mod stats;

use crate::{color::*, data::DataFrame, plot::BarLinePlot};

fn main() {
    let csv_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test_data.csv");
    let df = DataFrame::from_csv(csv_path).unwrap();
    let example_threads = *df
        .col("threads")
        .last()
        .expect("example fixture must contain at least one row");

    let df = df
        .filter(|r| r["threads"] == example_threads)
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
