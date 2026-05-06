use crate::{color::Color, data::GroupedFrame, ir::*, plot::group_stats};

struct LineSeries {
    column: String,
    label: String,
    color: Color,
}

pub struct LinePlot {
    df: GroupedFrame,
    series: Vec<LineSeries>,
    xlabel: String,
    ylabel: String,
    ymin: Option<f64>,
    xtick_labels: Option<Vec<String>>,
}

impl LinePlot {
    /// Create a new `LinePlot` over the given grouped data.
    pub fn new(df: GroupedFrame) -> Self {
        LinePlot {
            df,
            series: Vec::new(),
            xlabel: String::new(),
            ylabel: String::new(),
            ymin: Some(0.0),
            xtick_labels: None,
        }
    }

    /// Add a data series with an explicit preamble color selector.
    pub fn series(
        mut self,
        col: impl Into<String>,
        color: Color,
        label: impl Into<String>,
    ) -> Self {
        self.series.push(LineSeries { column: col.into(), color, label: label.into() });
        self
    }

    /// Set the x-axis label.
    pub fn xlabel(mut self, label: impl Into<String>) -> Self {
        self.xlabel = label.into();
        self
    }

    /// Set the y-axis label.
    pub fn ylabel(mut self, label: impl Into<String>) -> Self {
        self.ylabel = label.into();
        self
    }

    /// Set the minimum y value (`ymin`).  Pass `None` to let pgfplots choose.
    pub fn ymin(mut self, v: Option<f64>) -> Self {
        self.ymin = v;
        self
    }

    /// Override x-tick labels.  The length must match the number of groups.
    pub fn xtick_labels(mut self, labels: Vec<impl Into<String>>) -> Self {
        self.xtick_labels = Some(labels.into_iter().map(|l| l.into()).collect());
        self
    }

    /// Render to a TikZ `tikzpicture` string.
    pub fn render(&self) -> String {
        self.build_document().render_tikz()
    }

    fn build_document(&self) -> PlotDocument {
        let color_names: Vec<String> = (0..self.series.len())
            .map(|i| self.series[i].color.tikz_name().to_owned())
            .collect();

        PlotDocument {
            setup_lines: Vec::new(),
            ax0: self.build_axis(&color_names),
            ax1: None,
        }
    }

    fn build_axis(&self, color_names: &[String]) -> Axis {
        let mut options = crate::plot::common_axis_options(false);

        options.push(AxisOption::key_value("width", "\\epyfigurewidth"));
        options.push(AxisOption::key_value("height", "\\epyfigureheight"));
        if !self.xlabel.is_empty() {
            options.push(AxisOption::key_value(
                "xlabel",
                format!("{{\\epylabelsize {}}}", self.xlabel),
            ));
        }
        if !self.ylabel.is_empty() {
            options.push(AxisOption::key_value(
                "ylabel",
                format!("{{\\epylabelsize {}}}", self.ylabel),
            ));
        }
        if let Some(v) = self.ymin {
            options.push(AxisOption::key_value("ymin", v.to_string()));
        }

        // x-tick configuration
        let keys = self.df.keys();
        let n = keys.len();
        if n > 0 {
            let tick_str: Vec<String> = (0..n).map(|i| i.to_string()).collect();
            options.push(AxisOption::key_value("xtick", format!("{{{}}}", tick_str.join(","))));

            let labels: Vec<String> = if let Some(ref lbls) = self.xtick_labels {
                lbls.clone()
            } else {
                keys.iter().map(|&k| k.to_string()).collect()
            };
            options.push(AxisOption::key_value(
                "xticklabels",
                format!("{{{}}}", labels.join(",")),
            ));
        }

        // Extend axis limits by half a bar width on each side.
        if n > 0 {
            options.push(AxisOption::key_value("xmin", "-0.5"));
            options.push(AxisOption::key_value("xmax", (n as f64 - 0.5).to_string()));
        }

        let mut elements = Vec::new();

        // ── Series ────────────────────────────────────────────────────────
        for (si, series) in self.series.iter().enumerate() {
            let stats = group_stats(&self.df, &series.column);
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

        Axis { options, elements }
    }
}
