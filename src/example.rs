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

use crate::{data::DataFrame, plot::TwinPlot};

fn main() {
    let csv_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test_data.csv");
    let df = DataFrame::from_csv(csv_path).unwrap();
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

    // Twin-axis bar (efficiency) + line (throughput) plot
    let tikz = TwinPlot::new(grouped)
        .bar("gflop_j", r"GFLOP/J")
        .line("gflop_s", r"GFLOP/s")
        .xlabel(r"Power limit (W)")
        .render();

    std::fs::write("example_tikz.tex", tikz).unwrap();
}
