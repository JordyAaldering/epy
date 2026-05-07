mod color;
mod ir;
mod plot;

pub mod prelude {
    pub use polars::prelude::*;
    pub use crate::color::Color;
    pub use crate::plot::{TwinPlot, LinePlot, ZPlot};
}

use prelude::*;

/// Load the CSV and compute all derived columns in one lazy pass.
fn load_data() -> DataFrame {
    LazyCsvReader::new("test_data.csv".into())
        .with_has_header(true)
        .finish()
        .unwrap()
        .with_columns([
            // powercap is stored in µW; convert to W
            (col("powercap").cast(DataType::Float64) / lit(1_000_000.0f64))
                .alias("powercap_w"),
            (col("insns").cast(DataType::Float64) / col("runtime") / lit(1e9f64))
                .alias("gflop_s"),
            (col("insns").cast(DataType::Float64) / col("rapl") / lit(1e9f64))
                .alias("gflop_j"),
            (col("insns").cast(DataType::Float64) / col("cycs").cast(DataType::Float64))
                .alias("ipc"),
        ])
        .collect()
        .unwrap()
}

/// Return the maximum thread count present in the data.
fn max_threads(df: &DataFrame) -> i64 {
    df.column("threads").unwrap().i64().unwrap().max().unwrap()
}

fn twin(df: &DataFrame) {
    let filtered = df.clone().lazy()
        .filter(col("threads").eq(lit(max_threads(df))))
        .collect()
        .unwrap();

    let tikz = TwinPlot::new(
            filtered,
            "powercap_w",
            "gflop_j", "GFLOP/J",
            "gflop_s", "GFLOP/s",
            "Power limit (W)",
        )
        .build_document()
        .render_tikz();

    std::fs::write(".build/example_twin.tex", tikz).unwrap();
}

fn ipc(df: &DataFrame) {
    let filtered = df.clone().lazy()
        .filter(col("threads").eq(lit(max_threads(df))))
        .collect()
        .unwrap();

    let tikz = LinePlot::new(filtered, "powercap_w", "Power limit (W)", "IPC")
        .series("ipc", "IPC", Color::Runtime)
        .build_document()
        .annot_area((3.0 + 3.5) / 2.0, 0.0, (7.125 + 7.75) / 2.0, 1.0, Color::Annot)
        .annot_label(7.0, 0.25, "Hello, world!")
        .render_tikz();

    std::fs::write(".build/example_ipc.tex", tikz).unwrap();
}

fn zplot(df: &DataFrame) {
    let tikz = ZPlot::new(
            df.clone(),
            "threads", "powercap_w",
            "gflop_s", "gflop_j",
            "GFLOP/s", "GFLOP/J",
        )
        .build_document()
        .render_tikz();

    std::fs::write(".build/example_zplot.tex", tikz).unwrap();
}

fn main() {
    std::fs::create_dir_all(".build").unwrap();
    let df = load_data();
    twin(&df);
    ipc(&df);
    zplot(&df);
}
