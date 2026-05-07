//! Plot builders for energy-measurement data.
mod line;
mod twin;
mod zplot;

use std::collections::HashSet;

pub use line::LinePlot;
pub use twin::TwinPlot;
pub use zplot::ZPlot;

use crate::{color::Color, ir::*};
use polars::prelude::*;

pub(crate) fn common_axis_options() -> HashSet<AxisOption> {
    HashSet::from([
        AxisOption::ScaleOnlyAxis,
        AxisOption::AxisLineColor(Color::Grid),
        AxisOption::YGridColor(Color::Grid),
        AxisOption::YMajorGrids(true),
        AxisOption::MajorTickLength(Numeric::new(3.0)),
        AxisOption::XTickStyle(Style::new().with_color(Color::Grid)),
        AxisOption::YTickStyle(Style::new().with_color(Color::Grid)),
        AxisOption::TickLabelStyle(Style::new().with_font("\\epyticksize").with_inner_sep_pt(2.0)),
        AxisOption::LegendStyle(
            Style::new()
                .with_font("\\epylegendsize")
                .with_draw(Color::Grid)
                .with_fill_opacity(0.8)
                .with_draw_opacity(1.0)
                .with_text_opacity(1.0),
        ),
        AxisOption::EnsureAxisHeightExtraYTick,
        AxisOption::EnsureAxisHeightExtraYTickLabels,
        AxisOption::EnsureAxisHeightExtraYTickStyle,
    ])
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
