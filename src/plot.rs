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
        AxisOption::TickAlignOutside,
        AxisOption::XTickPosLeft,
        AxisOption::YTickPosLeft,
        AxisOption::YMajorGrids(true),
        AxisOption::MajorTickLength(Numeric::new(2.0)),
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

pub(crate) struct Quartiles {
    pub median: f64,
    pub q1: f64,
    pub q3: f64,
}

pub(crate) fn quartiles(xs: &[f64]) -> Quartiles {
    assert!(!xs.is_empty());

    let mut xs = xs.to_vec();
    xs.sort_by(f64::total_cmp);

    fn slice_median(xs: &[f64]) -> f64 {
        let n = xs.len();
        if n % 2 == 0 {
            (xs[n / 2 - 1] + xs[n / 2]) / 2.0
        } else {
            xs[n / 2]
        }
    }

    let n = xs.len();
    let median = slice_median(&xs);
    let q1 = slice_median(&xs[..n / 2]);
    let q3 = slice_median(&xs[(n + 1) / 2..]);
    Quartiles { median, q1, q3 }
}
