use crate::{data::GroupedFrame, plot::{fmt_f, group_stats}, ir::*};

/// Extra multiplier applied to the data maximum when estimating the longest
/// right-axis tick label.  pgfplots rounds the axis maximum up to the next
/// "nice" tick value, so the actual label is slightly larger than the data
/// maximum; 10 % is a conservative overshoot that covers most cases.
const TICK_ESTIMATE_BUFFER: f64 = 1.1;

pub struct TwinPlot {
    df: GroupedFrame,
    bar_column: Option<String>,
    bar_label: String,
    line_column: Option<String>,
    line_label: String,
    xlabel: String,
    xtick_labels: Option<Vec<String>>,
}

impl TwinPlot {
    pub fn new(df: GroupedFrame) -> Self {
        TwinPlot {
            df,
            bar_column: None,
            bar_label: String::new(),
            line_column: None,
            line_label: String::new(),
            xlabel: String::new(),
            xtick_labels: None,
        }
    }

    /// Configure the bar series (left y-axis). Uses `epyenergycolor`.
    pub fn bar(mut self, col: impl Into<String>, label: impl Into<String>) -> Self {
        self.bar_column = Some(col.into());
        self.bar_label = label.into();
        self
    }

    /// Configure the line series (right y-axis). Uses `epyruntimecolor`.
    pub fn line(mut self, col: impl Into<String>, label: impl Into<String>) -> Self {
        self.line_column = Some(col.into());
        self.line_label = label.into();
        self
    }

    /// Set the x-axis label (shared by both axes).
    pub fn xlabel(mut self, label: impl Into<String>) -> Self {
        self.xlabel = label.into();
        self
    }

    /// Override the x-tick labels (must match the number of groups).
    pub fn xtick_labels(mut self, labels: Vec<impl Into<String>>) -> Self {
        self.xtick_labels = Some(labels.into_iter().map(|l| l.into()).collect());
        self
    }

    /// Render to a TikZ `tikzpicture` string.
    pub fn render(&self) -> String {
        self.build_document().render_tikz()
    }

    fn build_document(&self) -> PlotDocument {
        let bar_col = self.bar_column.as_deref().unwrap_or_default();
        let line_col = self.line_column.as_deref().unwrap_or_default();

        let mut axes = vec![self.build_left_axis(bar_col, line_col)];
        if !line_col.is_empty() {
            axes.push(self.build_right_axis(line_col));
        }

        PlotDocument {
            setup_lines: self.twin_setup_lines(),
            axes,
        }
    }

    /// Return the maximum q3 value across all groups for the right (line) axis.
    fn max_line_value(&self) -> f64 {
        let col = self.line_column.as_deref().unwrap_or_default();
        if col.is_empty() {
            return 1.0;
        }
        let stats = group_stats(&self.df, col);
        stats.iter().map(|s| s.q3).fold(0.0_f64, f64::max)
    }

    /// Generate LaTeX length-setup commands that measure the right-axis padding
    /// at compile time, based on the longest expected tick label and the ylabel.
    ///
    /// The approach:
    /// - `\settowidth` measures the rendered width of the estimated longest tick
    ///   label using `\eplabelfont` — the same font macro used in the axis styles —
    ///   so the measurement automatically tracks any font-size change.
    /// - `\settoheight` measures one `\eplabelfont` line height, which equals the
    ///   horizontal footprint of the rotated ylabel.
    /// - Fixed overhead accounts for tick length (3 pt), tick-label inner sep
    ///   (2 pt), and the gap between tick labels and the ylabel (~5 pt).
    fn twin_setup_lines(&self) -> Vec<String> {
        // Multiply max by TICK_ESTIMATE_BUFFER so the estimate covers the "nice"
        // tick that pgfplots rounds up to above the data maximum.
        let tick_estimate = fmt_f(self.max_line_value() * TICK_ESTIMATE_BUFFER);
        let has_ylabel = !self.line_label.is_empty();

        let mut lines = Vec::new();

        // \epyticksize is defined in the preamble and matches the font used for
        // tick labels, so this measurement stays correct if the font changes.
        lines.push(format!(
            "\\settowidth{{\\epyrpad}}{{\\normalfont\\epyticksize {tick_estimate}}}"
        ));

        if has_ylabel {
            // The right ylabel is rotated 90°; its horizontal footprint equals one
            // \epyticksize line height.
            lines.push("\\settoheight{\\epyrlabelh}{\\normalfont\\epyticksize Ag}".to_owned());
            lines.push("\\addtolength{\\epyrpad}{\\epyrlabelh}".to_owned());
            // tick length (3pt) + inner sep (2pt) + gap to ylabel (~5pt)
            lines.push("\\addtolength{\\epyrpad}{10pt}".to_owned());
        } else {
            // tick length (3pt) + inner sep (2pt)
            lines.push("\\addtolength{\\epyrpad}{5pt}".to_owned());
        }

        lines
    }

    fn x_range(&self) -> (f64, f64) {
        let n = self.df.num_groups();
        (-0.5, n as f64 - 0.5)
    }

    fn xtick_str(&self) -> String {
        let n = self.df.num_groups();
        (0..n).map(|i| i.to_string()).collect::<Vec<_>>().join(",")
    }

    fn xticklabels_str(&self) -> String {
        let keys = self.df.keys();
        if let Some(ref lbls) = self.xtick_labels {
            lbls.join(",")
        } else {
            keys.iter().map(|&k| fmt_f(k)).collect::<Vec<_>>().join(",")
        }
    }

    fn build_left_axis(&self, bar_col: &str, line_col: &str) -> Axis {
        let (xmin, xmax) = self.x_range();
        let mut options = crate::plot::common_axis_options(false);
        options.push(AxisOption::key_value("name", "mainaxis"));
        options.push(AxisOption::flag("trim axis right"));

        // ── Axis options ──────────────────────────────────────────────────
        options.push(AxisOption::key_value("width", "{\\dimexpr \\epyfigurewidth - \\epyrpad\\relax}"));
        options.push(AxisOption::key_value("height", "\\epyfigureheight"));
        if !self.xlabel.is_empty() {
            options.push(AxisOption::key_value(
                "xlabel",
                format!("{{\\epylabelsize {}}}", self.xlabel),
            ));
        }
        if !self.bar_label.is_empty() {
            options.push(AxisOption::key_value(
                "ylabel",
                format!("{{\\epylabelsize {}}}", self.bar_label),
            ));
        }
        options.push(AxisOption::key_value("ymin", "0"));
        options.push(AxisOption::key_value("xmin", fmt_f(xmin)));
        options.push(AxisOption::key_value("xmax", fmt_f(xmax)));
        options.push(AxisOption::key_value("xtick", format!("{{{}}}", self.xtick_str())));
        options.push(AxisOption::key_value(
            "xticklabels",
            format!("{{{}}}", self.xticklabels_str()),
        ));

        let mut elements = Vec::new();

        // ── Bar series ────────────────────────────────────────────────────
        if !bar_col.is_empty() {
            let stats = group_stats(&self.df, bar_col);

            // Filled bars (median height)
            let mut bar_coordinates = Vec::new();
            for (i, s) in stats.iter().enumerate() {
                bar_coordinates.push(Coordinate::Plain(i as f64, s.median));
            }
            elements.push(AxisElement::Plot(AddPlot {
                options: vec![
                    "ybar".into(),
                    "bar width=0.7".into(),
                    "fill=epyenergycolor".into(),
                    "draw=none".into(),
                    "area legend".into(),
                ],
                coordinates: bar_coordinates,
                closed_cycle: false,
            }));
            elements.push(AxisElement::LegendEntry(self.bar_label.clone()));

            // Simple error bars: vertical whiskers from Q1 to Q3.
            for (i, s) in stats.iter().enumerate() {
                elements.push(AxisElement::DrawLine {
                    options: vec!["black!60".into(), "line width=0.9pt".into()],
                    from: Coordinate::AxisCs(i as f64, s.q1),
                    to: Coordinate::AxisCs(i as f64, s.q3),
                });
            }
        }

        // ── Legend placeholder for the right-axis line series ─────────────
        if !line_col.is_empty() && !self.line_label.is_empty() {
            elements.push(AxisElement::LegendImage(vec![
                "epyruntimecolor".into(),
                "mark=*".into(),
                "mark size=2pt".into(),
                "line width=1pt".into(),
            ]));
            elements.push(AxisElement::LegendEntry(self.line_label.clone()));
        }

        Axis { options, elements }
    }

    fn build_right_axis(&self, line_col: &str) -> Axis {
        let (xmin, xmax) = self.x_range();
        let stats = group_stats(&self.df, line_col);
        let mut options = crate::plot::common_axis_options(false);

        options.push(AxisOption::key_value("at", "{(mainaxis.south west)}"));
        options.push(AxisOption::key_value("anchor", "south west"));
        options.push(AxisOption::flag("trim axis left"));
        options.push(AxisOption::key_value("axis x line", "none"));
        options.push(AxisOption::key_value("xmajorgrids", "false"));
        options.push(AxisOption::key_value("ymajorgrids", "false"));
        options.push(AxisOption::key_value("xtick", "\\empty"));
        options.push(AxisOption::key_value("xticklabels", "\\empty"));

        options.push(AxisOption::key_value("axis y line", "right"));
        options.push(AxisOption::key_value("width", "{\\dimexpr \\epyfigurewidth - \\epyrpad\\relax}"));
        options.push(AxisOption::key_value("height", "\\epyfigureheight"));
        if !self.line_label.is_empty() {
            options.push(AxisOption::key_value(
                "ylabel",
                format!("{{\\epylabelsize {}}}", self.line_label),
            ));
        }
        options.push(AxisOption::key_value("ymin", "0"));
        options.push(AxisOption::key_value("xmin", fmt_f(xmin)));
        options.push(AxisOption::key_value("xmax", fmt_f(xmax)));

        let mut band_coordinates = Vec::new();

        for (i, s) in stats.iter().enumerate() {
            band_coordinates.push(Coordinate::Plain(i as f64, s.q3));
        }
        for (i, s) in stats.iter().enumerate().rev() {
            band_coordinates.push(Coordinate::Plain(i as f64, s.q1));
        }

        let mut line_coordinates = Vec::new();
        for (i, s) in stats.iter().enumerate() {
            line_coordinates.push(Coordinate::Plain(i as f64, s.median));
        }

        Axis {
            options,
            elements: vec![
                AxisElement::Plot(AddPlot {
                    options: vec![
                        "fill=epyruntimecomplementary".into(),
                        "draw=none".into(),
                        "forget plot".into(),
                    ],
                    coordinates: band_coordinates,
                    closed_cycle: true,
                }),
                AxisElement::Plot(AddPlot {
                    options: vec![
                        "epyruntimecolor".into(),
                        "mark=*".into(),
                        "mark size=2pt".into(),
                        "line width=1pt".into(),
                    ],
                    coordinates: line_coordinates,
                    closed_cycle: false,
                }),
            ],
        }
    }
}
