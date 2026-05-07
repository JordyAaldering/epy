use crate::{data::GroupedFrame, plot::group_stats, ir::*};

/// Extra multiplier applied to the data maximum when estimating the longest
/// right-axis tick label.  pgfplots rounds the axis maximum up to the next
/// "nice" tick value, so the actual label is slightly larger than the data
/// maximum; 10 % is a conservative overshoot that covers most cases.
const TICK_ESTIMATE_BUFFER: f64 = 1.1;

pub struct TwinPlot {
    df: GroupedFrame,
    bar_col: String,
    bar_label: String,
    line_col: String,
    line_label: String,
    xaxis_label: String,
    xtick_labels: Option<Vec<String>>,
}

impl TwinPlot {
    pub fn new(
        df: GroupedFrame,
        bar_col: &str,
        bar_label: &str,
        line_col: &str,
        line_label: &str,
        xaxis_label: &str,
    ) -> Self {
        TwinPlot {
            df,
            bar_col: bar_col.into(),
            bar_label: bar_label.into(),
            line_col: line_col.into(),
            line_label: line_label.into(),
            xaxis_label: xaxis_label.into(),
            xtick_labels: None,
        }
    }

    pub fn xtick_labels(mut self, labels: Vec<impl Into<String>>) -> Self {
        assert_eq!(labels.len(), self.df.num_groups());
        self.xtick_labels = Some(labels.into_iter().map(|l| l.into()).collect());
        self
    }

    pub fn build_document(&self) -> PlotDocument {
        let setup_lines = self.twin_setup_lines();
        let ax1 = self.build_left_axis();
        let ax2 = self.build_right_axis();
        PlotDocument::new(setup_lines, ax1, Some(ax2))
    }

    /// Return the maximum q3 value across all groups for the right (line) axis.
    fn max_line_value(&self) -> f64 {
        let stats = group_stats(&self.df, &self.line_col);
        stats.iter().map(|s| s.q3).fold(0.0_f64, f64::max)
    }

    fn twin_setup_lines(&self) -> Vec<String> {
        // Multiply max by TICK_ESTIMATE_BUFFER so the estimate covers the "nice"
        // tick that pgfplots rounds up to above the data maximum.
        let tick_estimate = (self.max_line_value() * TICK_ESTIMATE_BUFFER).to_string();
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
            keys.iter().map(|&k| k.to_string()).collect::<Vec<_>>().join(",")
        }
    }

    fn build_left_axis(&self) -> Axis {
        let (xmin, xmax) = self.x_range();
        let mut opts = crate::plot::common_axis_options();
        opts.push(AxisOption::key_value("name", "mainaxis"));
        opts.push(AxisOption::flag("trim axis right"));

        // ── Axis options ──────────────────────────────────────────────────
        opts.push(AxisOption::key_value("width", "{\\dimexpr \\epyfigurewidth - \\epyrpad\\relax}"));
        opts.push(AxisOption::key_value("height", "\\epyfigureheight"));
        opts.push(AxisOption::key_value("xlabel", format!("{{\\epylabelsize {}}}", self.xaxis_label)));
        opts.push(AxisOption::key_value("ylabel", format!("{{\\epylabelsize {}}}", self.bar_label)));
        opts.push(AxisOption::key_value("ymin", "0"));
        opts.push(AxisOption::key_value("xmin", xmin.to_string()));
        opts.push(AxisOption::key_value("xmax", xmax.to_string()));
        opts.push(AxisOption::key_value("xtick", format!("{{{}}}", self.xtick_str())));
        opts.push(AxisOption::key_value("xticklabels", format!("{{{}}}", self.xticklabels_str())));

        let mut elements = Vec::new();

        let stats = group_stats(&self.df, &self.bar_col);

        // Filled bars (median height)
        let mut bar_coordinates = Vec::new();
        for (i, s) in stats.iter().enumerate() {
            bar_coordinates.push(Coordinate::Plain(i as f64, s.median));
        }
        elements.push(AxisElement::Plot(AddPlot {
            opts: vec![
                "ybar".into(),
                "bar width=0.7".into(),
                "fill=epyenergycolor".into(),
                "draw=none".into(),
                "area legend".into(),
            ],
            coords: bar_coordinates,
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
        let (xmin, xmax) = self.x_range();
        let stats = group_stats(&self.df, &self.line_col);
        let mut opts = crate::plot::common_axis_options();

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
            opts,
            elements: vec![
                AxisElement::Plot(AddPlot {
                    opts: vec![
                        "fill=epyruntimecompl".into(),
                        "draw=none".into(),
                    ],
                    coords: band_coordinates,
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
                    coords: line_coordinates,
                    closed_cycle: false,
                }),
            ],
        }
    }
}
