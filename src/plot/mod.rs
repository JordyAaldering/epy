//! Plot builders for energy-measurement data.

pub mod bar_line;
pub mod line;
pub mod zplot;

pub use bar_line::BarLinePlot;
pub use line::LinePlot;
pub use zplot::ZPlot;

use crate::color::Color;
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

/// Emit `\definecolor` lines for the colors used in a plot.
pub(crate) fn emit_color_defs(colors: &[(&Color, &str)]) -> String {
    colors
        .iter()
        .map(|(c, name)| c.define(name))
        .collect::<Vec<_>>()
        .join("\n")
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

/// Wrap TikZ axis content in a `tikzpicture`.
pub(crate) fn wrap_tikzpicture(color_defs: &str, body: &str) -> String {
    let mut out = String::from("\\begin{tikzpicture}\n");
    if !color_defs.is_empty() {
        out.push_str(color_defs);
        out.push('\n');
        out.push('\n');
    }
    out.push_str(body);
    out.push_str("\\end{tikzpicture}\n");
    out
}
