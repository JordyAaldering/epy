mod line_grouped;
mod line;
mod twin;
mod zplot;

pub use line_grouped::LineGrouped;
pub use line::LinePlot;
pub use twin::TwinPlot;
pub use zplot::ZPlot;

use crate::ir::*;

use std::collections::HashSet;

pub(crate) fn common_axis_options() -> HashSet<AxisOption> {
    HashSet::from([
        AxisOption::AxisLineColor(GRID_COLOR.into()),
        AxisOption::YGridStyle(
            Style::new()
                .with_color(GRID_COLOR.into())
                .with_line_width_pt(0.4),
        ),
        AxisOption::TickAlignOutside,
        AxisOption::XTickPosLeft,
        AxisOption::YTickPosLeft,
        AxisOption::YMajorGrids(true),
        AxisOption::MajorTickLength(Numeric::new(MAJOR_TICK_LENGTH_EM)),
        AxisOption::XTickStyle(
            Style::new()
                .with_color(GRID_COLOR.into())
                .with_line_width_pt(0.4),
        ),
        AxisOption::YTickStyle(
            Style::new()
                .with_color(GRID_COLOR.into())
                .with_line_width_pt(0.4),
        ),
        AxisOption::TickLabelStyle(Style::new()
            .with_inner_sep_em(TICK_LABEL_INNER_SEP_EM)
            .with_outer_sep_em(TICK_LABEL_OUTER_SEP_EM)
        ),
        AxisOption::LegendStyle(
            Style::new()
                .with_draw(GRID_COLOR.into())
                .with_fill_opacity(0.9)
                .with_draw_opacity(1.0)
                .with_text_opacity(1.0),
        ),
        AxisOption::EnsureAxisHeightExtraYTick,
        AxisOption::EnsureAxisHeightExtraYTickLabels,
        AxisOption::EnsureAxisHeightExtraYTickStyle,
        AxisOption::ScaledTicksFalse,
        AxisOption::TickNumberFormatFixed,
    ])
}

pub struct Quartiles {
    pub median: f64,
    pub q1: f64,
    pub q3: f64,
}

pub fn median(xs: &[f64]) -> f64 {
    let n = xs.len();
    if n % 2 == 0 {
        (xs[n / 2 - 1] + xs[n / 2]) / 2.0
    } else {
        xs[n / 2]
    }
}

pub fn quartiles(xs: &[f64]) -> Quartiles {
    assert!(!xs.is_empty());

    let mut xs = xs.to_vec();
    xs.sort_by(f64::total_cmp);

    let n = xs.len();
    let m = median(&xs);
    let q1 = median(&xs[..n / 2]);
    let q3 = median(&xs[(n + 1) / 2..]);
    Quartiles { median: m, q1, q3 }
}
