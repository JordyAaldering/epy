//! Simple line plot with optional IQR confidence band.
//!
//! Generates a single `\begin{axis}...\end{axis}` block styled with the
//! project's `common/line` pgfplots style.

use crate::color::Color;
use crate::data::GroupedFrame;
use crate::plot::{emit_color_defs, fmt_f, group_stats, wrap_tikzpicture};

// ── LineSeries ────────────────────────────────────────────────────────────

struct LineSeries {
    col: String,
    color: Color,
    label: String,
}

// ── LinePlot ──────────────────────────────────────────────────────────────

/// A line plot with median values and an IQR shaded band.
///
/// # Example
/// ```no_run
/// use energy_plots::prelude::*;
///
/// let df = DataFrame::from_csv("data.csv").unwrap()
///     .filter(|r| r["threads"] == 8.0)
///     .with_column("ipc", |r| r["insns"] / r["cycs"]);
///
/// let tikz = LinePlot::new(df.group_by("powercap"))
///     .series("ipc", palette::BLUE, "IPC")
///     .xlabel(r"Power limit (\si{\watt})")
///     .ylabel("IPC")
///     .render();
///
/// std::fs::write("plot.tex", tikz).unwrap();
/// ```
pub struct LinePlot {
    grouped: GroupedFrame,
    series: Vec<LineSeries>,
    xlabel: String,
    ylabel: String,
    height_ratio: f64,
    ymin: Option<f64>,
    xtick_labels: Option<Vec<String>>,
}

impl LinePlot {
    /// Create a new `LinePlot` over the given grouped data.
    pub fn new(grouped: GroupedFrame) -> Self {
        LinePlot {
            grouped,
            series: Vec::new(),
            xlabel: String::new(),
            ylabel: String::new(),
            height_ratio: 0.8,
            ymin: Some(0.0),
            xtick_labels: None,
        }
    }

    /// Add a data series.
    ///
    /// * `col`   – column name in the underlying data frame
    /// * `color` – line and fill color
    /// * `label` – legend label (may contain LaTeX)
    pub fn series(mut self, col: impl Into<String>, color: Color, label: impl Into<String>) -> Self {
        self.series.push(LineSeries { col: col.into(), color, label: label.into() });
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

    /// Override the height as a fraction of `\linewidth` (default: `0.8`).
    pub fn height_ratio(mut self, r: f64) -> Self {
        self.height_ratio = r;
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
        let color_names: Vec<String> = (0..self.series.len())
            .map(|i| format!("epColor{i}"))
            .collect();
        let color_defs: Vec<(&Color, &str)> = self
            .series
            .iter()
            .enumerate()
            .map(|(i, s)| (&s.color, color_names[i].as_str()))
            .collect();

        let defs = emit_color_defs(&color_defs);
        let body = self.render_axis(&color_names);
        wrap_tikzpicture(&defs, &body)
    }

    fn render_axis(&self, color_names: &[String]) -> String {
        let mut out = String::new();

        // ── Axis options ──────────────────────────────────────────────────
        out.push_str("\\begin{axis}[\n");
        out.push_str("  common/line,\n");
        out.push_str("  width=\\linewidth,\n");
        out.push_str(&format!("  height={}\\linewidth,\n", fmt_f(self.height_ratio)));
        if !self.xlabel.is_empty() {
            out.push_str(&format!("  xlabel={{{}}},\n", self.xlabel));
        }
        if !self.ylabel.is_empty() {
            out.push_str(&format!("  ylabel={{{}}},\n", self.ylabel));
        }
        if let Some(v) = self.ymin {
            out.push_str(&format!("  ymin={},\n", fmt_f(v)));
        }

        // x-tick configuration
        let keys = self.grouped.keys();
        let n = keys.len();
        if n > 0 {
            let tick_str: Vec<String> = (0..n).map(|i| i.to_string()).collect();
            out.push_str(&format!("  xtick={{{}}},\n", tick_str.join(",")));

            let labels: Vec<String> = if let Some(ref lbls) = self.xtick_labels {
                lbls.clone()
            } else {
                keys.iter().map(|&k| fmt_f(k)).collect()
            };
            out.push_str(&format!("  xticklabels={{{}}},\n", labels.join(",")));
        }

        // Extend axis limits by half a bar width on each side.
        if n > 0 {
            out.push_str(&format!("  xmin={},\n", fmt_f(-0.5)));
            out.push_str(&format!("  xmax={},\n", fmt_f(n as f64 - 0.5)));
        }

        out.push_str("]\n");

        // ── Series ────────────────────────────────────────────────────────
        for (si, series) in self.series.iter().enumerate() {
            let stats = group_stats(&self.grouped, &series.col);
            let cn = &color_names[si];

            // IQR band (closed polygon: upper half forward, lower half backward)
            out.push_str(&format!(
                "\\addplot[{cn}!40!white, opacity=0.3, draw=none, forget plot]\n  coordinates {{\n"
            ));
            for (i, s) in stats.iter().enumerate() {
                out.push_str(&format!("    ({}, {})\n", i, fmt_f(s.q3)));
            }
            for (i, s) in stats.iter().enumerate().rev() {
                out.push_str(&format!("    ({}, {})\n", i, fmt_f(s.q1)));
            }
            out.push_str("  } \\closedcycle;\n");

            // Median line
            out.push_str(&format!(
                "\\addplot[{cn}, mark=*, mark size=2pt, line width=1pt]\n  coordinates {{\n"
            ));
            for (i, s) in stats.iter().enumerate() {
                out.push_str(&format!("    ({}, {})\n", i, fmt_f(s.median)));
            }
            out.push_str("  };\n");

            // Legend
            out.push_str(&format!("\\addlegendentry{{{}}}\n", series.label));
        }

        out.push_str("\\end{axis}\n");
        out
    }
}
