use polars::prelude::*;
use crate::{ir::*, plot::{common_axis_options, format_key, series_to_f64}};

/// Extra multiplier applied to the data maximum when estimating the longest
/// right-axis tick label.  pgfplots rounds the axis maximum up to the next
/// "nice" tick value; 10 % is a conservative overshoot that covers most cases.
const TICK_ESTIMATE_BUFFER: f64 = 1.1;

pub struct TwinPlot {
    df: DataFrame,
    group_col: String,
    bar_col: String,
    bar_label: String,
    line_col: String,
    line_label: String,
    xaxis_label: String,
    xtick_labels: Option<Vec<String>>,
}

impl TwinPlot {
    pub fn new(
        df: DataFrame,
        group_col: &str,
        bar_col: &str,
        bar_label: &str,
        line_col: &str,
        line_label: &str,
        xaxis_label: &str,
    ) -> Self {
        TwinPlot {
            df,
            group_col: group_col.into(),
            bar_col: bar_col.into(),
            bar_label: bar_label.into(),
            line_col: line_col.into(),
            line_label: line_label.into(),
            xaxis_label: xaxis_label.into(),
            xtick_labels: None,
        }
    }

    pub fn xtick_labels(mut self, labels: Vec<impl Into<String>>) -> Self {
        self.xtick_labels = Some(labels.into_iter().map(|l| l.into()).collect());
        self
    }

    pub fn build_document(&self) -> PlotDocument {
        let setup_lines = self.twin_setup_lines();
        let ax1 = self.build_left_axis();
        let ax2 = self.build_right_axis();
        PlotDocument::new(setup_lines, ax1, Some(ax2))
    }

    fn stats_columns(&self, value_col: &str) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
        let result = self.df.clone().lazy()
            .group_by([polars::prelude::col(&self.group_col)])
            .agg([
                polars::prelude::col(value_col).median().alias("_med"),
                polars::prelude::col(value_col).quantile(polars::prelude::lit(0.25_f64), polars::prelude::QuantileMethod::Linear).alias("_q1"),
                polars::prelude::col(value_col).quantile(polars::prelude::lit(0.75_f64), polars::prelude::QuantileMethod::Linear).alias("_q3"),
            ])
            .sort_by_exprs([polars::prelude::col(&self.group_col)], polars::prelude::SortMultipleOptions::default())
            .collect()
            .expect("TwinPlot aggregation failed");

        (
            series_to_f64(result.column(&self.group_col).unwrap()),
            series_to_f64(result.column("_med").unwrap()),
            series_to_f64(result.column("_q1").unwrap()),
            series_to_f64(result.column("_q3").unwrap()),
        )
    }

    fn max_line_q3(&self) -> f64 {
        let (_, _, _, q3s) = self.stats_columns(&self.line_col);
        q3s.into_iter().fold(0.0_f64, f64::max)
    }

    fn twin_setup_lines(&self) -> Vec<String> {
        let tick_estimate = (self.max_line_q3() * TICK_ESTIMATE_BUFFER).to_string();
        let has_ylabel = !self.line_label.is_empty();

        let mut lines = Vec::new();
        lines.push(format!(
            "\\settowidth{{\\epyrpad}}{{\\normalfont\\epyticksize {tick_estimate}}}"
        ));
        if has_ylabel {
            lines.push("\\settoheight{\\epyrlabelh}{\\normalfont\\epyticksize Ag}".to_owned());
            lines.push("\\addtolength{\\epyrpad}{\\epyrlabelh}".to_owned());
            lines.push("\\addtolength{\\epyrpad}{10pt}".to_owned());
        } else {
            lines.push("\\addtolength{\\epyrpad}{5pt}".to_owned());
        }
        lines
    }

    fn x_range(&self, n: usize) -> (f64, f64) {
        (-0.5, n as f64 - 0.5)
    }

    fn xtick_str(&self, n: usize) -> String {
        (0..n).map(|i| i.to_string()).collect::<Vec<_>>().join(",")
    }

    fn xticklabels_str(&self, keys: &[f64]) -> String {
        if let Some(ref lbls) = self.xtick_labels {
            lbls.join(",")
        } else {
            keys.iter().map(|&k| format_key(k)).collect::<Vec<_>>().join(",")
        }
    }

    fn build_left_axis(&self) -> Axis {
        let (keys, meds, q1s, q3s) = self.stats_columns(&self.bar_col);
        let n = keys.len();
        let (xmin, xmax) = self.x_range(n);

        let mut opts = common_axis_options();
        opts.push(AxisOption::key_value("name", "mainaxis"));
        opts.push(AxisOption::flag("trim axis right"));
        opts.push(AxisOption::key_value("width", "{\\dimexpr \\epyfigurewidth - \\epyrpad\\relax}"));
        opts.push(AxisOption::key_value("height", "\\epyfigureheight"));
        opts.push(AxisOption::key_value("xlabel", format!("{{\\epylabelsize {}}}", self.xaxis_label)));
        opts.push(AxisOption::key_value("ylabel", format!("{{\\epylabelsize {}}}", self.bar_label)));
        opts.push(AxisOption::key_value("ymin", "0"));
        opts.push(AxisOption::key_value("xmin", xmin.to_string()));
        opts.push(AxisOption::key_value("xmax", xmax.to_string()));
        opts.push(AxisOption::key_value("xtick", format!("{{{}}}", self.xtick_str(n))));
        opts.push(AxisOption::key_value("xticklabels", format!("{{{}}}", self.xticklabels_str(&keys))));

        let mut elements = Vec::new();

        // Filled bars (median height)
        let bar_coords: Vec<Coordinate> = meds.iter().enumerate()
            .map(|(i, median)| Coordinate::Plain(i as f64, *median))
            .collect();
        elements.push(AxisElement::Plot(AddPlot {
            opts: vec![
                "ybar".into(),
                "bar width=0.7".into(),
                "fill=epyenergycolor".into(),
                "draw=none".into(),
                "area legend".into(),
            ],
            coords: bar_coords,
            closed_cycle: false,
        }));
        elements.push(AxisElement::LegendEntry(self.bar_label.clone()));

        // Error whiskers Q1–Q3
        for i in 0..q1s.len() {
            elements.push(AxisElement::DrawLine {
                options: vec!["black!60".into(), "line width=0.9pt".into()],
                from: Coordinate::AxisCs(i as f64, q1s[i]),
                to: Coordinate::AxisCs(i as f64, q3s[i]),
            });
        }

        // Legend image + entry for the right-axis line series
        elements.push(AxisElement::LegendImage(vec![
            "epyruntimecolor".into(),
            "mark=*".into(),
            "mark options={solid,draw=white}".into(),
            "mark size=2pt".into(),
            "line width=1pt".into(),
        ]));
        elements.push(AxisElement::LegendEntry(self.line_label.clone()));

        Axis { opts, elements }
    }

    fn build_right_axis(&self) -> Axis {
        let (keys, meds, q1s, q3s) = self.stats_columns(&self.line_col);
        let n = keys.len();
        let (xmin, xmax) = self.x_range(n);

        let mut opts = common_axis_options();
        opts.push(AxisOption::key_value("at", "{(mainaxis.south west)}"));
        opts.push(AxisOption::key_value("anchor", "south west"));
        opts.push(AxisOption::flag("trim axis left"));
        opts.push(AxisOption::key_value("axis x line", "none"));
        opts.push(AxisOption::key_value("xmajorgrids", "false"));
        opts.push(AxisOption::key_value("ymajorgrids", "false"));
        opts.push(AxisOption::key_value("xtick", "\\empty"));
        opts.push(AxisOption::key_value("xticklabels", "\\empty"));
        opts.push(AxisOption::key_value("axis y line", "right"));
        opts.push(AxisOption::key_value("width", "{\\dimexpr \\epyfigurewidth - \\epyrpad\\relax}"));
        opts.push(AxisOption::key_value("height", "\\epyfigureheight"));
        opts.push(AxisOption::key_value("ylabel", format!("{{\\epylabelsize {}}}", self.line_label)));
        opts.push(AxisOption::key_value("ymin", "0"));
        opts.push(AxisOption::key_value("xmin", xmin.to_string()));
        opts.push(AxisOption::key_value("xmax", xmax.to_string()));

        let mut band = Vec::new();
        for (i, q3) in q3s.iter().enumerate() {
            band.push(Coordinate::Plain(i as f64, *q3));
        }
        for (i, q1) in q1s.iter().enumerate().rev() {
            band.push(Coordinate::Plain(i as f64, *q1));
        }

        let line: Vec<Coordinate> = meds.iter().enumerate()
            .map(|(i, median)| Coordinate::Plain(i as f64, *median))
            .collect();

        Axis {
            opts,
            elements: vec![
                AxisElement::Plot(AddPlot {
                    opts: vec!["fill=epyruntimecompl".into(), "draw=none".into(), "forget plot".into()],
                    coords: band,
                    closed_cycle: true,
                }),
                AxisElement::Plot(AddPlot {
                    opts: vec![
                        "epyruntimecolor".into(),
                        "mark=*".into(),
                        "mark options={solid,draw=white}".into(),
                        "mark size=2pt".into(),
                        "line width=1pt".into(),
                    ],
                    coords: line,
                    closed_cycle: false,
                }),
            ],
        }
    }
}
