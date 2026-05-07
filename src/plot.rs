//! Plot builders for energy-measurement data.
mod line;
mod twin;
mod zplot;

pub use line::LinePlot;
pub use twin::TwinPlot;
pub use zplot::ZPlot;

use crate::{color::Color, ir::*};
use polars::prelude::*;

pub(crate) fn common_axis_options() -> Vec<AxisOption> {
    vec![
        AxisOption::flag("scale only axis"),
        AxisOption::key_value("axis line style", format!("{{{}}}", Color::Grid.tikz_name())),
        AxisOption::key_value("y grid style", format!("{{{}}}", Color::Grid.tikz_name())),
        AxisOption::flag("ymajorgrids"),
        AxisOption::key_value("major tick length", "3pt"),
        AxisOption::key_value("xtick style", format!("{{color={}}}", Color::Grid.tikz_name())),
        AxisOption::key_value("ytick style", format!("{{color={}}}", Color::Grid.tikz_name())),
        AxisOption::key_value("tick label style", r"{font=\epyticksize, inner sep=2pt}"),
        AxisOption::key_value("legend style", format!("{{font=\\epylegendsize,draw={},fill opacity=0.8,draw opacity=1,text opacity=1}}", Color::Grid.tikz_name())),
        AxisOption::key_value("extra y ticks", r"{\pgfkeysvalueof{/pgfplots/ymax}}"),
        AxisOption::key_value("extra y tick labels", r"{\vphantom{Ag}}"),
        AxisOption::key_value("extra y tick style", "{yticklabel style={opacity=0,text opacity=0},major tick length=0pt}"),
    ]
}

/// Cast a polars `Column` to `Vec<f64>`, skipping any nulls.
pub(crate) fn series_to_f64(c: &Column) -> Vec<f64> {
    c.cast(&DataType::Float64)
        .unwrap()
        .as_series()
        .unwrap()
        .f64()
        .unwrap()
        .into_no_null_iter()
        .collect()
}

/// Format a float key for a tick label: whole-number values are rendered
/// without a decimal point (e.g. `5.0` → `"5"`).
pub(crate) fn format_key(k: f64) -> String {
    if k.fract() == 0.0 && k.abs() < 1e15 {
        format!("{}", k as i64)
    } else {
        format!("{k}")
    }
}
