//! # energy-plots
//!
//! A Rust DSL for generating clean PGFPlots/TikZ code from CSV energy
//! measurement data.
//!
//! ## Overview
//!
//! The library is built around three plot types that cover the main
//! energy-measurement use cases:
//!
//! | Type | Struct | Use case |
//! |------|--------|----------|
//! | Bar + line (twin axes) | [`BarLinePlot`] | Energy efficiency (bar, left) vs. throughput (line, right) |
//! | Line with IQR band | [`LinePlot`] | IPC or other derived metrics across power caps |
//! | Z-plot (scatter) | [`ZPlot`] | Efficiency vs. throughput for multiple configurations |
//!
//! All plots compute the **median** and **IQR** (Q1 / Q3) across repeated
//! measurements automatically.
//!
//! ## Quick start
//!
//! ```no_run
//! use energy_plots::prelude::*;
//!
//! // Load and prepare data
//! let df = DataFrame::from_csv("results/prog.csv").unwrap()
//!     .filter(|r| r["threads"] == 8.0)
//!     .with_column("gflop_j", |r| r["insns"] / r["rapl"] / 1e9)
//!     .with_column("gflop_s", |r| r["insns"] / r["runtime"] / 1e9);
//!
//! let grouped = df.group_by("powercap");
//!
//! // Generate bar+line twin-axis plot
//! let tikz = BarLinePlot::new(grouped)
//!     .bar("gflop_j", palette::GREEN, r"\si{\giga\flop\per\joule}")
//!     .line("gflop_s", palette::RED, r"\si{\giga\flop\per\second}")
//!     .xlabel(r"Power limit (\si{\watt})")
//!     .render();
//!
//! std::fs::write("plot.tex", tikz).unwrap();
//! ```
//!
//! ## LaTeX prelude
//!
//! The generated code relies on the `common/*` pgfplots styles defined in the
//! project LaTeX prelude (see `README.md`).  Include the prelude in your
//! document before `\input`-ting any generated `.tex` file.

pub mod color;
pub mod data;
pub mod plot;
pub mod stats;

pub use color::Color;
pub use data::{DataFrame, GroupedFrame};
pub use plot::{BarLinePlot, LinePlot, ZPlot};

/// Convenience re-exports for typical usage.
pub mod prelude {
    pub use crate::color::{Color, palette};
    pub use crate::data::{DataFrame, GroupedFrame};
    pub use crate::plot::{BarLinePlot, LinePlot, ZPlot};
    pub use crate::stats::{mean, median, q1, q3};
}
