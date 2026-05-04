//! Twin-axis bar+line plot.
//!
//! The left y-axis shows a bar chart (typically energy efficiency, GFLOP/J).
//! The right y-axis shows a line plot (typically throughput, GFLOP/s).
//! Both axes share the same x-axis (typically power cap in watts).
//!
//! Uses the project's `common/twin-main` and `common/twin` pgfplots styles to
//! ensure consistent sizing and alignment without the margin calculation issues
//! that arise when using matplot2tikz with `twinx` axes.

use crate::color::Color;
use crate::data::GroupedFrame;
use crate::plot::{emit_color_defs, fmt_f, group_stats};

// ── BarLinePlot ───────────────────────────────────────────────────────────

/// Extra multiplier applied to the data maximum when estimating the longest
/// right-axis tick label.  pgfplots rounds the axis maximum up to the next
/// "nice" tick value, so the actual label is slightly larger than the data
/// maximum; 10 % is a conservative overshoot that covers most cases.
const TICK_ESTIMATE_BUFFER: f64 = 1.1;

/// A combined bar (left y-axis) and line (right y-axis) twin-axis plot.
///
/// # Example
/// ```no_run
/// use energy_plots::prelude::*;
///
/// let df = DataFrame::from_csv("data.csv").unwrap()
///     .filter(|r| r["threads"] == 8.0)
///     .with_column("gflop_j", |r| r["insns"] / r["rapl"] / 1e9)
///     .with_column("gflop_s", |r| r["insns"] / r["runtime"] / 1e9);
///
/// let grouped = df.group_by("powercap");
/// let tikz = BarLinePlot::new(grouped)
///     .bar("gflop_j", palette::GREEN, r"\si{\giga\flop\per\joule}")
///     .line("gflop_s", palette::RED, r"\si{\giga\flop\per\second}")
///     .xlabel(r"Power limit (\si{\watt})")
///     .render();
///
/// std::fs::write("plot.tex", tikz).unwrap();
/// ```
pub struct BarLinePlot {
    grouped: GroupedFrame,
    bar_col: Option<String>,
    bar_color: Color,
    bar_label: String,
    line_col: Option<String>,
    line_color: Color,
    line_label: String,
    xlabel: String,
    height_ratio: f64,
    bar_width: f64,
    xtick_labels: Option<Vec<String>>,
}

impl BarLinePlot {
    /// Create a new `BarLinePlot` over the given grouped data.
    pub fn new(grouped: GroupedFrame) -> Self {
        BarLinePlot {
            grouped,
            bar_col: None,
            bar_color: crate::color::palette::GREEN,
            bar_label: String::new(),
            line_col: None,
            line_color: crate::color::palette::RED,
            line_label: String::new(),
            xlabel: String::new(),
            height_ratio: 0.8,
            bar_width: 0.7,
            xtick_labels: None,
        }
    }

    /// Configure the bar series (left y-axis).
    pub fn bar(mut self, col: impl Into<String>, color: Color, label: impl Into<String>) -> Self {
        self.bar_col = Some(col.into());
        self.bar_color = color;
        self.bar_label = label.into();
        self
    }

    /// Configure the line series (right y-axis).
    pub fn line(mut self, col: impl Into<String>, color: Color, label: impl Into<String>) -> Self {
        self.line_col = Some(col.into());
        self.line_color = color;
        self.line_label = label.into();
        self
    }

    /// Set the x-axis label (shared by both axes).
    pub fn xlabel(mut self, label: impl Into<String>) -> Self {
        self.xlabel = label.into();
        self
    }

    /// Override the plot height as a fraction of `\linewidth` (default: `0.8`).
    pub fn height_ratio(mut self, r: f64) -> Self {
        self.height_ratio = r;
        self
    }

    /// Override the bar width in axis units (default: `0.7`).
    pub fn bar_width(mut self, w: f64) -> Self {
        self.bar_width = w;
        self
    }

    /// Override the x-tick labels (must match the number of groups).
    pub fn xtick_labels(mut self, labels: Vec<impl Into<String>>) -> Self {
        self.xtick_labels = Some(labels.into_iter().map(|l| l.into()).collect());
        self
    }

    /// Render to a TikZ `tikzpicture` string.
    pub fn render(&self) -> String {
        let bar_col = self.bar_col.as_deref().unwrap_or_default();
        let line_col = self.line_col.as_deref().unwrap_or_default();

        let defs = emit_color_defs(&[
            (&self.bar_color, "epBar"),
            (&self.line_color, "epLine"),
        ]);

        let setup = self.render_twin_setup();
        let left = self.render_left_axis(bar_col, line_col);
        let right = self.render_right_axis(line_col);

        let mut out = String::from("\\begin{tikzpicture}\n");
        if !defs.is_empty() {
            out.push_str(&defs);
            out.push_str("\n\n");
        }
        out.push_str(&setup);
        out.push('\n');
        out.push_str(&left);
        out.push('\n');
        out.push_str(&right);
        out.push_str("\\end{tikzpicture}\n");
        out
    }

    /// Return the maximum q3 value across all groups for the right (line) axis.
    fn max_line_value(&self) -> f64 {
        let col = self.line_col.as_deref().unwrap_or_default();
        if col.is_empty() {
            return 1.0;
        }
        let stats = group_stats(&self.grouped, col);
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
    fn render_twin_setup(&self) -> String {
        // Multiply max by TICK_ESTIMATE_BUFFER so the estimate covers the "nice"
        // tick that pgfplots rounds up to above the data maximum.
        let tick_estimate = fmt_f(self.max_line_value() * TICK_ESTIMATE_BUFFER);
        let has_ylabel = !self.line_label.is_empty();

        let mut out = String::new();

        // Guard against duplicate \newlength when the file is \input more than once.
        out.push_str("\\ifdefined\\epRpad\\else\\newlength{\\epRpad}\\fi\n");
        // \eplabelfont is defined in the preamble and matches the font used in
        // the axis styles, so this measurement stays correct if the font changes.
        out.push_str(&format!(
            "\\settowidth{{\\epRpad}}{{\\eplabelfont {tick_estimate}}}\n"
        ));

        if has_ylabel {
            // The right ylabel is rotated 90°; its horizontal footprint equals one
            // \eplabelfont line height.
            out.push_str("\\ifdefined\\epRlabelH\\else\\newlength{\\epRlabelH}\\fi\n");
            out.push_str("\\settoheight{\\epRlabelH}{\\eplabelfont Ag}\n");
            out.push_str("\\addtolength{\\epRpad}{\\epRlabelH}\n");
            // tick length (3pt) + inner sep (2pt) + gap to ylabel (~5pt)
            out.push_str("\\addtolength{\\epRpad}{10pt}\n");
        } else {
            // tick length (3pt) + inner sep (2pt)
            out.push_str("\\addtolength{\\epRpad}{5pt}\n");
        }

        out
    }

    fn x_range(&self) -> (f64, f64) {
        let n = self.grouped.num_groups();
        (-0.5, n as f64 - 0.5)
    }

    fn xtick_str(&self) -> String {
        let n = self.grouped.num_groups();
        (0..n).map(|i| i.to_string()).collect::<Vec<_>>().join(",")
    }

    fn xticklabels_str(&self) -> String {
        let keys = self.grouped.keys();
        if let Some(ref lbls) = self.xtick_labels {
            lbls.join(",")
        } else {
            keys.iter().map(|&k| fmt_f(k)).collect::<Vec<_>>().join(",")
        }
    }

    fn render_left_axis(&self, bar_col: &str, line_col: &str) -> String {
        let mut out = String::new();
        let (xmin, xmax) = self.x_range();

        // ── Axis options ──────────────────────────────────────────────────
        out.push_str("\\begin{axis}[\n");
        out.push_str("  common/twin-main, common/bar,\n");
        out.push_str("  width={\\dimexpr \\linewidth - \\epRpad\\relax},\n");
        out.push_str(&format!("  height={}\\linewidth,\n", fmt_f(self.height_ratio)));
        if !self.xlabel.is_empty() {
            out.push_str(&format!("  xlabel={{{}}},\n", self.xlabel));
        }
        if !self.bar_label.is_empty() {
            out.push_str(&format!("  ylabel={{{}}},\n", self.bar_label));
        }
        out.push_str("  ymin=0,\n");
        out.push_str(&format!("  xmin={}, xmax={},\n", fmt_f(xmin), fmt_f(xmax)));
        out.push_str(&format!("  xtick={{{}}},\n", self.xtick_str()));
        out.push_str(&format!("  xticklabels={{{}}},\n", self.xticklabels_str()));
        out.push_str("]\n");

        // ── Bar series ────────────────────────────────────────────────────
        if !bar_col.is_empty() {
            let stats = group_stats(&self.grouped, bar_col);

            // Filled bars (median height)
            out.push_str(&format!(
                "\\addplot[ybar, bar width={}, fill=epBar, draw=none]\n  coordinates {{\n",
                fmt_f(self.bar_width)
            ));
            for (i, s) in stats.iter().enumerate() {
                out.push_str(&format!("    ({i}, {})\n", fmt_f(s.median)));
            }
            out.push_str("  };\n");
            out.push_str(&format!("\\addlegendentry{{{}}}\n", self.bar_label));

            // IQR whiskers (vertical line from Q1 to Q3)
            for (i, s) in stats.iter().enumerate() {
                out.push_str(&format!(
                    "\\draw[black!60, line width=0.9pt] (axis cs:{i},{}) -- (axis cs:{i},{});\n",
                    fmt_f(s.q1),
                    fmt_f(s.q3)
                ));
            }
        }

        // ── Legend placeholder for the right-axis line series ─────────────
        if !line_col.is_empty() && !self.line_label.is_empty() {
            out.push_str("\\addlegendimage{epLine, mark=*, mark size=2pt, line width=1pt}\n");
            out.push_str(&format!("\\addlegendentry{{{}}}\n", self.line_label));
        }

        out.push_str("\\end{axis}\n");
        out
    }

    fn render_right_axis(&self, line_col: &str) -> String {
        if line_col.is_empty() {
            return String::new();
        }

        let mut out = String::new();
        let (xmin, xmax) = self.x_range();
        let stats = group_stats(&self.grouped, line_col);

        // ── Axis options ──────────────────────────────────────────────────
        out.push_str("\\begin{axis}[\n");
        out.push_str("  common/twin, common/line,\n");
        out.push_str("  axis y line=right,\n");
        out.push_str("  width={\\dimexpr \\linewidth - \\epRpad\\relax},\n");
        out.push_str(&format!("  height={}\\linewidth,\n", fmt_f(self.height_ratio)));
        if !self.line_label.is_empty() {
            out.push_str(&format!("  ylabel={{{}}},\n", self.line_label));
        }
        out.push_str("  ymin=0,\n");
        out.push_str(&format!("  xmin={}, xmax={},\n", fmt_f(xmin), fmt_f(xmax)));
        out.push_str("]\n");

        // ── IQR band ──────────────────────────────────────────────────────
        out.push_str("\\addplot[epLine!40!white, opacity=0.3, draw=none, forget plot]\n  coordinates {\n");
        for (i, s) in stats.iter().enumerate() {
            out.push_str(&format!("    ({i}, {})\n", fmt_f(s.q3)));
        }
        for (i, s) in stats.iter().enumerate().rev() {
            out.push_str(&format!("    ({i}, {})\n", fmt_f(s.q1)));
        }
        out.push_str("  } \\closedcycle;\n");

        // ── Median line ───────────────────────────────────────────────────
        out.push_str("\\addplot[epLine, mark=*, mark size=2pt, line width=1pt]\n  coordinates {\n");
        for (i, s) in stats.iter().enumerate() {
            out.push_str(&format!("    ({i}, {})\n", fmt_f(s.median)));
        }
        out.push_str("  };\n");

        out.push_str("\\end{axis}\n");
        out
    }
}
