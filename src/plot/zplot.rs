//! Z-plot: efficiency (GFLOP/J) vs. throughput (GFLOP/s) for multiple
//! program configurations, grouped by a categorical column such as thread count.
//!
//! Each group becomes a separate series with a distinct color and marker.

use crate::color::{Color, palette};
use crate::data::GroupedFrame;
use crate::plot::{emit_color_defs, fmt_f, wrap_tikzpicture};

// ── Marker shapes ─────────────────────────────────────────────────────────

const MARKERS: &[&str] = &["*", "square*", "triangle*", "diamond*", "pentagon*", "o", "square", "triangle"];

// ── ZPlot ─────────────────────────────────────────────────────────────────

/// A scatter / Z-plot of efficiency vs. throughput, with one series per group.
///
/// The x-axis is usually throughput (GFLOP/s) and the y-axis is efficiency
/// (GFLOP/J).  Multiple groups (e.g. different thread counts) are drawn as
/// separate series so the trade-offs between configurations are visually clear.
///
/// # Example
/// ```no_run
/// use energy_plots::prelude::*;
///
/// let df = DataFrame::from_csv("data.csv").unwrap()
///     .with_column("gflop_j", |r| r["insns"] / r["rapl"] / 1e9)
///     .with_column("gflop_s", |r| r["insns"] / r["runtime"] / 1e9);
///
/// let tikz = ZPlot::new(df.group_by("threads"))
///     .x_col("gflop_s")
///     .y_col("gflop_j")
///     .xlabel(r"\si{\giga\flop\per\second}")
///     .ylabel(r"\si{\giga\flop\per\joule}")
///     .label_fn(|tc| format!("{} threads", tc as u32))
///     .render();
///
/// std::fs::write("zplot.tex", tikz).unwrap();
/// ```
pub struct ZPlot {
    grouped: GroupedFrame,
    x_col: String,
    y_col: String,
    xlabel: String,
    ylabel: String,
    height_ratio: f64,
    label_fn: Box<dyn Fn(f64) -> String>,
    colors: Option<Vec<Color>>,
}

impl ZPlot {
    /// Create a new `ZPlot` grouped by the given frame.
    ///
    /// The grouping column determines the series separation (e.g. thread count).
    pub fn new(grouped: GroupedFrame) -> Self {
        ZPlot {
            grouped,
            x_col: String::new(),
            y_col: String::new(),
            xlabel: String::new(),
            ylabel: String::new(),
            height_ratio: 0.8,
            label_fn: Box::new(|k| format!("{k}")),
            colors: None,
        }
    }

    /// Column to use as the x-axis values (one value per row within each group).
    pub fn x_col(mut self, col: impl Into<String>) -> Self {
        self.x_col = col.into();
        self
    }

    /// Column to use as the y-axis values.
    pub fn y_col(mut self, col: impl Into<String>) -> Self {
        self.y_col = col.into();
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

    /// Override the plot height as a fraction of `\linewidth` (default: `0.8`).
    pub fn height_ratio(mut self, r: f64) -> Self {
        self.height_ratio = r;
        self
    }

    /// Custom function to turn a group key into a legend label.
    ///
    /// Defaults to simply formatting the key as a number.
    pub fn label_fn(mut self, f: impl Fn(f64) -> String + 'static) -> Self {
        self.label_fn = Box::new(f);
        self
    }

    /// Override the series colors (length must be ≥ number of groups).
    pub fn colors(mut self, colors: Vec<Color>) -> Self {
        self.colors = Some(colors);
        self
    }

    /// Render to a TikZ `tikzpicture` string.
    pub fn render(&self) -> String {
        let n = self.grouped.num_groups();
        let default_palette = &palette::SERIES[..];
        let colors: &[Color] = self
            .colors
            .as_deref()
            .unwrap_or(default_palette);

        // Build color definitions
        let color_names: Vec<String> = (0..n).map(|i| format!("epSeries{i}")).collect();
        let color_pairs: Vec<(&Color, &str)> = (0..n)
            .map(|i| (&colors[i % colors.len()], color_names[i].as_str()))
            .collect();
        let defs = emit_color_defs(&color_pairs);

        let body = self.render_axis(&color_names);
        wrap_tikzpicture(&defs, &body)
    }

    fn render_axis(&self, color_names: &[String]) -> String {
        let mut out = String::new();
        let n = self.grouped.num_groups();

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
        out.push_str("  ymin=0,\n");
        out.push_str("]\n");

        // ── Series (one per group) ─────────────────────────────────────────
        for gi in 0..n {
            let key = self.grouped.unique_keys[gi];
            let label = (self.label_fn)(key);
            let cn = &color_names[gi];
            let marker = MARKERS[gi % MARKERS.len()];

            // For each sub-group within this group, compute mean x and mean y.
            // Here the "sub-group" is just all rows in this group – the caller
            // is expected to have pre-filtered or pre-aggregated as needed.
            let xs = self.grouped.group_values(gi, &self.x_col);
            let ys = self.grouped.group_values(gi, &self.y_col);

            // If the group has multiple rows we plot each point individually
            // (the caller should call `group_by` on an already-aggregated frame,
            // or use a secondary grouping via `DataFrame::group_by` twice).
            out.push_str(&format!(
                "\\addplot[{cn}, mark={marker}, mark size=2pt, line width=1pt]\n  coordinates {{\n"
            ));
            for (&x, &y) in xs.iter().zip(ys.iter()) {
                out.push_str(&format!("    ({}, {})\n", fmt_f(x), fmt_f(y)));
            }
            out.push_str("  };\n");
            out.push_str(&format!("\\addlegendentry{{{label}}}\n"));
        }

        out.push_str("\\end{axis}\n");
        out
    }
}
