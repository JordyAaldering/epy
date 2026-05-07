use crate::{color::Color, data::GroupedFrame, ir::*, plot::{common_axis_options, group_stats}};

struct LineSeries {
    col: String,
    label: String,
    color: Color,
}

pub struct LinePlot {
    df: GroupedFrame,
    series: Vec<LineSeries>,
    xaxis_label: String,
    yaxis_label: String,
    ymin: Option<f64>,
    xtick_labels: Option<Vec<String>>,
}

impl LinePlot {
    pub fn new(
        df: GroupedFrame,
        xaxis_label: &str,
        yaxis_label: &str,
    ) -> Self {
        LinePlot {
            df,
            series: Vec::new(),
            xaxis_label: xaxis_label.into(),
            yaxis_label: yaxis_label.into(),
            ymin: Some(0.0),
            xtick_labels: None,
        }
    }

    pub fn series(
        mut self,
        col: &str,
        label: &str,
        color: Color,
    ) -> Self {
        self.series.push(LineSeries { col: col.into(), label: label.into(), color });
        self
    }

    pub fn ymin(mut self, v: Option<f64>) -> Self {
        self.ymin = v;
        self
    }

    pub fn xtick_labels(mut self, labels: Vec<impl Into<String>>) -> Self {
        assert_eq!(labels.len(), self.df.num_groups());
        self.xtick_labels = Some(labels.into_iter().map(|l| l.into()).collect());
        self
    }

    pub fn build_document(&self) -> PlotDocument {
        let ax = self.build_axis();
        PlotDocument::new(Vec::new(), ax, None)
    }

    fn build_axis(&self) -> Axis {
        let color_names: Vec<String> = (0..self.series.len())
            .map(|i| self.series[i].color.tikz_name().to_owned())
            .collect();

        let mut opts = common_axis_options();
        opts.push(AxisOption::key_value("width", "\\epyfigurewidth"));
        opts.push(AxisOption::key_value("height", "\\epyfigureheight"));
        opts.push(AxisOption::key_value("xlabel", format!("{{\\epylabelsize {}}}", self.xaxis_label)));
        opts.push(AxisOption::key_value("ylabel", format!("{{\\epylabelsize {}}}", self.yaxis_label)));
        if let Some(v) = self.ymin {
            opts.push(AxisOption::key_value("ymin", v.to_string()));
        }

        let keys = self.df.keys();
        let n = keys.len();
        let tick_str: Vec<String> = (0..n).map(|i| i.to_string()).collect();
        opts.push(AxisOption::key_value("xtick", format!("{{{}}}", tick_str.join(","))));

        let labels: Vec<String> = if let Some(ref lbls) = self.xtick_labels {
            lbls.clone()
        } else {
            keys.iter().map(|&k| k.to_string()).collect()
        };
        opts.push(AxisOption::key_value("xticklabels", format!("{{{}}}", labels.join(","))));

        // Extend axis limits by half a bar width on each side.
        opts.push(AxisOption::key_value("xmin", "-0.5"));
        opts.push(AxisOption::key_value("xmax", (n as f64 - 0.5).to_string()));

        let mut elements = Vec::new();

        for (si, series) in self.series.iter().enumerate() {
            let stats = group_stats(&self.df, &series.col);
            let cn = &color_names[si];

            // Transparent Q1-Q3 band (upper half forward, lower half backward).
            let mut band_coordinates = Vec::new();
            for (i, s) in stats.iter().enumerate() {
                band_coordinates.push(Coordinate::Plain(i as f64, s.q3));
            }
            for (i, s) in stats.iter().enumerate().rev() {
                band_coordinates.push(Coordinate::Plain(i as f64, s.q1));
            }
            elements.push(AxisElement::Plot(AddPlot {
                options: vec![cn.clone(), "opacity=0.3".into(), "draw=none".into(), "forget plot".into()],
                coordinates: band_coordinates,
                closed_cycle: true,
            }));

            // Median line
            let mut line_coordinates = Vec::new();
            for (i, s) in stats.iter().enumerate() {
                line_coordinates.push(Coordinate::Plain(i as f64, s.median));
            }
            elements.push(AxisElement::Plot(AddPlot {
                options: vec![cn.clone(), "mark=*".into(), "mark size=2pt".into(), "line width=1pt".into()],
                coordinates: line_coordinates,
                closed_cycle: false,
            }));

            // Legend
            elements.push(AxisElement::LegendEntry(series.label.clone()));
        }

        Axis { opts, elements }
    }
}
