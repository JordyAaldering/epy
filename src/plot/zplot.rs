use polars::prelude::*;
use crate::{color::Color, ir::*, plot::{common_axis_options, format_key, series_to_f64}};

const MARKERS: &[&str] = &["*", "square*", "triangle*", "diamond*", "pentagon*", "o", "square", "triangle"];

pub struct ZPlot {
    df: DataFrame,
    /// Column whose unique sorted values define the legend series.
    series_col: String,
    /// Column used to group points within each series (the "hue" dimension).
    hue_col: String,
    x_col: String,
    y_col: String,
    xaxis_label: String,
    yaxis_label: String,
}

impl ZPlot {
    /// Create a scatter plot where each unique value of `series_col` becomes a
    /// separate series in the legend.  Within each series, rows are grouped by
    /// `hue_col` and the mean `(x_col, y_col)` is plotted per group.
    pub fn new(
        df: DataFrame,
        series_col: &str,
        hue_col: &str,
        x_col: &str,
        y_col: &str,
        xaxis_label: &str,
        yaxis_label: &str,
    ) -> Self {
        ZPlot {
            df,
            series_col: series_col.into(),
            hue_col: hue_col.into(),
            x_col: x_col.into(),
            y_col: y_col.into(),
            xaxis_label: xaxis_label.into(),
            yaxis_label: yaxis_label.into(),
        }
    }

    pub fn build_document(&self) -> PlotDocument {
        PlotDocument::new(Vec::new(), self.build_axis(), None)
    }

    fn build_axis(&self) -> Axis {
        // Single polars query: group by (series, hue), average x and y.
        // Sort by series first, then hue so we can scan in order.
        let agg = self.df.clone().lazy()
            .group_by([col(&self.series_col), col(&self.hue_col)])
            .agg([
                col(&self.x_col).mean().alias("_x"),
                col(&self.y_col).mean().alias("_y"),
            ])
            .sort_by_exprs(
                [col(&self.series_col), col(&self.hue_col)],
                SortMultipleOptions::default(),
            )
            .collect()
            .expect("ZPlot: polars aggregation failed");

        let sv = series_to_f64(agg.column(&self.series_col).unwrap());
        let xs = series_to_f64(agg.column("_x").unwrap());
        let ys = series_to_f64(agg.column("_y").unwrap());

        // Partition consecutive rows with the same series value into groups.
        let mut series_groups: Vec<(f64, Vec<Coordinate>)> = Vec::new();
        for i in 0..sv.len() {
            let key = sv[i];
            if series_groups.last().map_or(true, |(k, _)| k.to_bits() != key.to_bits()) {
                series_groups.push((key, Vec::new()));
            }
            series_groups.last_mut().unwrap().1.push(Coordinate::Plain(xs[i], ys[i]));
        }

        let mut opts = common_axis_options();
        opts.push(AxisOption::key_value("x grid style", format!("{{{}}}", Color::Grid.tikz_name())));
        opts.push(AxisOption::flag("xmajorgrids"));
        opts.push(AxisOption::key_value("width", "\\epyfigurewidth"));
        opts.push(AxisOption::key_value("height", "\\epyfigureheight"));
        opts.push(AxisOption::key_value("xlabel", format!("{{\\epylabelsize {}}}", self.xaxis_label)));
        opts.push(AxisOption::key_value("ylabel", format!("{{\\epylabelsize {}}}", self.yaxis_label)));
        opts.push(AxisOption::key_value("ymin", "0"));

        let mut elements = Vec::new();
        for (gi, (key, coords)) in series_groups.into_iter().enumerate() {
            let cn = Color::Colorblind(gi).tikz_name();
            let marker = MARKERS[gi % MARKERS.len()];
            elements.push(AxisElement::Plot(AddPlot {
                opts: vec![
                    cn.to_owned(),
                    format!("mark={marker}"),
                    "mark size=2pt".into(),
                    "mark options={solid,draw=white}".into(),
                    "line width=1pt".into(),
                ],
                coords,
                closed_cycle: false,
            }));
            elements.push(AxisElement::LegendEntry(format_key(key)));
        }

        Axis { opts, elements }
    }
}
