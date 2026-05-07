use polars::prelude::*;
use crate::{color::Color, ir::*, plot::{common_axis_options, format_key, series_to_f64}};

struct LineSeries {
    col: String,
    label: String,
    color: Color,
}

pub struct LinePlot {
    df: DataFrame,
    x_col: String,
    series: Vec<LineSeries>,
    xaxis_label: String,
    yaxis_label: String,
    ymin: Option<f64>,
    xtick_labels: Option<Vec<String>>,
}

impl LinePlot {
    pub fn new(df: DataFrame, x_col: &str, xaxis_label: &str, yaxis_label: &str) -> Self {
        LinePlot {
            df,
            x_col: x_col.into(),
            series: Vec::new(),
            xaxis_label: xaxis_label.into(),
            yaxis_label: yaxis_label.into(),
            ymin: Some(0.0),
            xtick_labels: None,
        }
    }

    pub fn series(mut self, col: &str, label: &str, color: Color) -> Self {
        self.series.push(LineSeries { col: col.into(), label: label.into(), color });
        self
    }

    pub fn ymin(mut self, v: Option<f64>) -> Self {
        self.ymin = v;
        self
    }

    pub fn xtick_labels(mut self, labels: Vec<String>) -> Self {
        self.xtick_labels = Some(labels);
        self
    }

    pub fn build_document(&self) -> PlotDocument {
        let ax = self.build_axis();
        PlotDocument::new(Vec::new(), ax, None)
    }

    fn build_axis(&self) -> Axis {
        // Compute stats for every series up-front; all share the same x grouping.
        let all_stats: Vec<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>)> = self.series.iter()
            .map(|s| {
                let result = self.df.clone().lazy()
                    .group_by([polars::prelude::col(&self.x_col)])
                    .agg([
                        polars::prelude::col(&s.col).median().alias("_med"),
                        polars::prelude::col(&s.col).quantile(polars::prelude::lit(0.25_f64), polars::prelude::QuantileMethod::Linear).alias("_q1"),
                        polars::prelude::col(&s.col).quantile(polars::prelude::lit(0.75_f64), polars::prelude::QuantileMethod::Linear).alias("_q3"),
                    ])
                    .sort_by_exprs([polars::prelude::col(&self.x_col)], polars::prelude::SortMultipleOptions::default())
                    .collect()
                    .expect("LinePlot aggregation failed");

                (
                    series_to_f64(result.column(&self.x_col).unwrap()),
                    series_to_f64(result.column("_med").unwrap()),
                    series_to_f64(result.column("_q1").unwrap()),
                    series_to_f64(result.column("_q3").unwrap()),
                )
            })
            .collect();

        let keys = all_stats.first().map(|(keys, _, _, _)| keys.clone()).unwrap_or_default();
        let n = keys.len();

        let mut opts = common_axis_options();
        opts.push(AxisOption::key_value("width", "\\epyfigurewidth"));
        opts.push(AxisOption::key_value("height", "\\epyfigureheight"));
        opts.push(AxisOption::key_value("xlabel", format!("{{\\epylabelsize {}}}", self.xaxis_label)));
        opts.push(AxisOption::key_value("ylabel", format!("{{\\epylabelsize {}}}", self.yaxis_label)));
        if let Some(v) = self.ymin {
            opts.push(AxisOption::key_value("ymin", v.to_string()));
        }

        let tick_str: Vec<String> = (0..n).map(|i| i.to_string()).collect();
        opts.push(AxisOption::key_value("xtick", format!("{{{}}}", tick_str.join(","))));

        let labels: Vec<String> = if let Some(ref lbls) = self.xtick_labels {
            lbls.clone()
        } else {
            keys.iter().map(|&k| format_key(k)).collect()
        };
        opts.push(AxisOption::key_value("xticklabels", format!("{{{}}}", labels.join(","))));
        opts.push(AxisOption::key_value("xmin", "-0.5"));
        opts.push(AxisOption::key_value("xmax", (n as f64 - 0.5).to_string()));

        let mut elements = Vec::new();

        for (si, series) in self.series.iter().enumerate() {
            let (_, meds, q1s, q3s) = &all_stats[si];
            let cn = series.color.tikz_name().to_owned();

            // Transparent Q1–Q3 band
            let mut band = Vec::new();
            for (i, q3) in q3s.iter().enumerate() {
                band.push(Coordinate::Plain(i as f64, *q3));
            }
            for (i, q1) in q1s.iter().enumerate().rev() {
                band.push(Coordinate::Plain(i as f64, *q1));
            }
            elements.push(AxisElement::Plot(AddPlot {
                opts: vec![cn.clone(), "opacity=0.3".into(), "draw=none".into(), "forget plot".into()],
                coords: band,
                closed_cycle: true,
            }));

            // Median line
            let line: Vec<Coordinate> = meds.iter().enumerate()
                .map(|(i, median)| Coordinate::Plain(i as f64, *median))
                .collect();
            elements.push(AxisElement::Plot(AddPlot {
                opts: vec![cn, "mark=*".into(), "mark size=2pt".into(), "line width=1pt".into()],
                coords: line,
                closed_cycle: false,
            }));

            elements.push(AxisElement::LegendEntry(series.label.clone()));
        }

        Axis { opts, elements }
    }
}
