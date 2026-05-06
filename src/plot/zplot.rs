use crate::{color::Color, data::GroupedFrame, ir::*, plot::fmt_f};

const MARKERS: &[&str] = &["*", "square*", "triangle*", "diamond*", "pentagon*", "o", "square", "triangle"];

/// A scatter / Z-plot of efficiency vs. throughput, with one series per group.
///
/// The x-axis is usually throughput (GFLOP/s) and the y-axis is efficiency
/// (GFLOP/J).  Multiple groups (e.g. different thread counts) are drawn as
/// separate series so the trade-offs between configurations are visually clear.
///
/// # Example
/// ```no_run
/// use epy::prelude::*;
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
            height_ratio: 1.0,
            label_fn: Box::new(|k| format!("{k}")),
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

    /// Override the plot height as a fraction of `\epyfigureheight` (default: `1.0`).
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

    /// Render to a TikZ `tikzpicture` string.
    pub fn render(&self) -> String {
        self.build_document().render_tikz()
    }

    fn build_document(&self) -> PlotDocument {
        let n = self.grouped.num_groups();
        // Colors are assumed to be predefined in the preamble as epycolorblind0, epycolorblind1, …
        let color_names: Vec<String> = (0..n).map(|i| Color::Colorblind(i).tikz_name()).collect();

        PlotDocument {
            setup_lines: Vec::new(),
            axes: vec![self.build_axis(&color_names)],
        }
    }

    fn build_axis(&self, color_names: &[String]) -> Axis {
        let n = self.grouped.num_groups();
        let mut options = crate::plot::common_axis_options(true);

        // ── Axis options ──────────────────────────────────────────────────
        options.push(AxisOption::key_value("width", "\\epyfigurewidth"));
        options.push(AxisOption::key_value(
            "height",
            format!("{}\\epyfigureheight", fmt_f(self.height_ratio)),
        ));
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
        options.push(AxisOption::key_value("ymin", "0"));

        let mut elements = Vec::new();

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
            let mut coordinates = Vec::new();
            for (&x, &y) in xs.iter().zip(ys.iter()) {
                coordinates.push(Coordinate::Plain(x, y));
            }
            elements.push(AxisElement::Plot(AddPlot {
                options: vec![cn.clone(), format!("mark={marker}"), "mark size=2pt".into(), "line width=1pt".into()],
                coordinates,
                closed_cycle: false,
            }));
            elements.push(AxisElement::LegendEntry(label));
        }

        Axis { options, elements }
    }
}
