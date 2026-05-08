mod line;
mod twin;
mod zplot;

use std::collections::HashSet;

pub use line::LinePlot;
pub use twin::TwinPlot;
pub use zplot::ZPlot;

use crate::{color::Color, ir::*};

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

pub(crate) fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

pub(crate) fn quantile_linear(mut values: Vec<f64>, q: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(f64::total_cmp);
    let n = values.len();
    if n == 1 {
        return values[0];
    }

    let pos = q.clamp(0.0, 1.0) * (n as f64 - 1.0);
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        values[lo]
    } else {
        let t = pos - lo as f64;
        values[lo] * (1.0 - t) + values[hi] * t
    }
}

pub(crate) fn median(values: Vec<f64>) -> f64 {
    quantile_linear(values, 0.5)
}
