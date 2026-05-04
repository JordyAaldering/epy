//! Plot builders for energy-measurement data.

pub mod bar_line;
mod ir;
pub mod line;
pub mod zplot;

pub use bar_line::BarLinePlot;
pub use line::LinePlot;
pub use zplot::ZPlot;

use crate::data::GroupedFrame;
use crate::stats::{GroupStats, q1, q3, median};

/// Compute per-group statistics for `col` from a [`GroupedFrame`].
pub(crate) fn group_stats(gf: &GroupedFrame, col: &str) -> Vec<GroupStats> {
    (0..gf.num_groups())
        .map(|gi| {
            let vals = gf.group_values(gi, col);
            GroupStats {
                x: gf.unique_keys[gi],
                median: median(&vals),
                q1: q1(&vals),
                q3: q3(&vals),
            }
        })
        .collect()
}

/// Format a float for TikZ coordinates: trim unnecessary trailing zeros.
pub(crate) fn fmt_f(v: f64) -> String {
    if v == 0.0 {
        return "0".to_owned();
    }
    // Use enough precision but strip trailing zeros.
    let s = format!("{v:.6}");
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_owned()
}
