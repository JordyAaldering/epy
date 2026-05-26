mod line;
mod timeseries;
mod twin;
mod zplot;

pub use line::LinePlot;
pub use timeseries::TimeSeries;
pub use twin::TwinPlot;
pub use zplot::ZPlot;

use crate::tikzir::*;

use ordermap::ordermap;

const GRID_COLOR: &'static str = "black!20";

pub(crate) const MARK_SIZE_PT: f64 = 2.0;
pub(crate) const MARK_OUTLINE_PT: f64 = MARK_SIZE_PT / 5.0;
pub(crate) const MARKERS: &[&str] = &[
    "*",
    "square*",
    "pentagon*",
    "diamond*",
    "triangle*",
    "halfcircle*",
    "halfsquare*",
    "halfdiamond*",
];

pub fn filter_every(i: usize, stride: usize) -> bool {
    i % stride == 0
}

pub fn filter_stride(i: usize, start: usize, stride: usize) -> bool {
    i >= start && (i - start) % stride == 0
}

pub fn format_precision(s: &str, precision: usize, trim: bool) -> String {
    if trim {
        format!("{:.precision$}", s.parse::<f64>().unwrap())
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        format!("{:.precision$}", s.parse::<f64>().unwrap())
    }
}

pub(crate) fn common_axis_options() -> Style {
    StyleBuilder::default()
        .width(Dimension::Code("\\epyfigurewidth".into()))
        .height(Dimension::Code("{\\epyheightratio*\\epyfigurewidth}".into()))
        .tick_align(TickAlign::Outside)
        .scaled_ticks(false)
        .xtick_pos(TickPos::Left)
        .ytick_pos(TickPos::Left)
        .y_major_grids(true)
        .major_tick_length(Dimension::Em(0.3))
        .legend_cell_align(CellAlign::Left)
        .extra_yticks(vec![Cs::Code("\\pgfkeysvalueof{/pgfplots/ymax}".into())])
        .extra_ytick_labels(vec!["\\vphantom{Ag}".into()])
        .number_format(NumberFormat::Fixed(false))
        .style_overrides(ordermap! {
            "axis line style".into() => StyleBuilder::default()
                .color(GRID_COLOR)
                .build()
                .unwrap(),
            "y grid style".into() => StyleBuilder::default()
                .color(GRID_COLOR)
                .line_width(Dimension::Pt(0.4))
                .build()
                .unwrap(),
            "xtick style".into() => StyleBuilder::default()
                .color(GRID_COLOR)
                .line_width(Dimension::Pt(0.4))
                .build()
                .unwrap(),
            "ytick style".into() => StyleBuilder::default()
                .color(GRID_COLOR)
                .line_width(Dimension::Pt(0.4))
                .build()
                .unwrap(),
            "tick label style".into() => StyleBuilder::default()
                .inner_sep(Dimension::Em(0.15))
                .build()
                .unwrap(),
            "ylabel style".into() => StyleBuilder::default()
                .outer_sep(Dimension::Em(-0.25))
                .build()
                .unwrap(),
            "extra y tick style".into() => StyleBuilder::default()
                .major_tick_length(Dimension::Pt(0.0))
                .grid(GridLines::None)
                .style_overrides(ordermap! {
                    "yticklabel style".into() => StyleBuilder::default()
                        .opacity(0.0)
                        .text_opacity(0.0)
                        .build()
                        .unwrap(),
                })
                .build()
                .unwrap(),
            "legend style".into() => StyleBuilder::default()
                .draw(GRID_COLOR)
                .inner_sep(Dimension::Em(0.2))
                .fill_opacity(0.9)
                .draw_opacity(1.0)
                .text_opacity(1.0)
                .build()
                .unwrap(),
        })
        .build()
        .unwrap()
}

pub struct Quartiles {
    pub median: f64,
    pub q1: f64,
    pub q3: f64,
}

pub fn median(xs: &[f64]) -> f64 {
    assert!(!xs.is_empty());
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
    if n < 3 {
        return Quartiles {
            median: m,
            q1: m,
            q3: m,
        };
    }

    let q1 = median(&xs[..n / 2]);
    let q3 = median(&xs[(n + 1) / 2..]);
    Quartiles { median: m, q1, q3 }
}
