//! Plot builders for energy-measurement data.
mod line;
mod twin;
mod zplot;

pub use line::LinePlot;
pub use twin::TwinPlot;
pub use zplot::ZPlot;

use crate::{data::GroupedFrame, ir::*, stats::*};

/// Return the axis options that every plot type sets directly in the generated
/// TikZ, replacing what was formerly in `\pgfplotsset`.
///
/// * `xgrid` — also enable x major grid lines (used for z-plots).
pub(crate) fn common_axis_options(xgrid: bool) -> Vec<AxisOption> {
    let mut opts = vec![
        AxisOption::flag("scale only axis"),
        AxisOption::key_value("axis line style", "{epygridcolor}"),
        AxisOption::key_value("y grid style", "{epygridcolor}"),
        AxisOption::flag("ymajorgrids"),
        AxisOption::key_value("major tick length", "3pt"),
        AxisOption::key_value("xtick style", "{color=epygridcolor}"),
        AxisOption::key_value("ytick style", "{color=epygridcolor}"),
        AxisOption::key_value("tick label style", r"{font=\epyticksize, inner sep=2pt}"),
        AxisOption::key_value(
            "legend style",
            r"{font=\epylegendsize, draw=epygridcolor, fill opacity=0.8, draw opacity=1, text opacity=1}",
        ),
    ];
    if xgrid {
        opts.push(AxisOption::key_value("x grid style", "{epygridcolor}"));
        opts.push(AxisOption::flag("xmajorgrids"));
    }
    opts
}

/// Compute per-group statistics for `col` from a [`GroupedFrame`].
pub(crate) fn group_stats(gf: &GroupedFrame, col: &str) -> Vec<IQR> {
    (0..gf.num_groups())
        .map(|gi| {
            let xs = gf.group_values(gi, col);
            IQR {
                median: median(&xs),
                q1: q1(&xs),
                q3: q3(&xs),
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
