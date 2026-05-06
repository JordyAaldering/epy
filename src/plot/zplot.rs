use crate::{color::Color, data::GroupedFrame, ir::*};

const MARKERS: &[&str] = &["*", "square*", "triangle*", "diamond*", "pentagon*", "o", "square", "triangle"];

pub struct ZPlot {
    df: GroupedFrame,
    x_column: String,
    y_column: String,
    xlabel: String,
    ylabel: String,
}

impl ZPlot {
    /// Create a new `ZPlot` grouped by the given frame.
    ///
    /// The grouping column determines the series separation (e.g. thread count).
    pub fn new(df: GroupedFrame) -> Self {
        ZPlot {
            df,
            x_column: String::new(),
            y_column: String::new(),
            xlabel: String::new(),
            ylabel: String::new(),
        }
    }

    /// Column to use as the x-axis values (one value per row within each group).
    pub fn x_col(mut self, col: impl Into<String>) -> Self {
        self.x_column = col.into();
        self
    }

    /// Column to use as the y-axis values.
    pub fn y_col(mut self, col: impl Into<String>) -> Self {
        self.y_column = col.into();
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

    /// Render to a TikZ `tikzpicture` string.
    pub fn render(&self) -> String {
        self.build_document().render_tikz()
    }

    fn build_document(&self) -> PlotDocument {
        let n = self.df.num_groups();
        // Colors are assumed to be predefined in the preamble as epycolorblind0, epycolorblind1, …
        let color_names: Vec<String> = (0..n).map(|i| Color::Colorblind(i).tikz_name()).collect();

        PlotDocument {
            setup_lines: Vec::new(),
            ax0: self.build_axis(&color_names),
            ax1: None,
        }
    }

    fn build_axis(&self, color_names: &[String]) -> Axis {
        let n = self.df.num_groups();
        let mut opts = crate::plot::common_axis_options();
        opts.push(AxisOption::key_value("x grid style", "{epygridcolor}"));
        opts.push(AxisOption::flag("xmajorgrids"));

        // ── Axis options ──────────────────────────────────────────────────
        opts.push(AxisOption::key_value("width", "\\epyfigurewidth"));
        opts.push(AxisOption::key_value("height", "\\epyfigureheight"));
        if !self.xlabel.is_empty() {
            opts.push(AxisOption::key_value(
                "xlabel",
                format!("{{\\epylabelsize {}}}", self.xlabel),
            ));
        }
        if !self.ylabel.is_empty() {
            opts.push(AxisOption::key_value(
                "ylabel",
                format!("{{\\epylabelsize {}}}", self.ylabel),
            ));
        }
        opts.push(AxisOption::key_value("ymin", "0"));

        let mut elements = Vec::new();

        for gi in 0..n {
            let key = self.df.unique_keys[gi];
            let label = key.to_string();
            let cn = &color_names[gi];
            let marker = MARKERS[gi % MARKERS.len()];

            // For each sub-group within this group, compute mean x and mean y.
            // Here the "sub-group" is just all rows in this group – the caller
            // is expected to have pre-filtered or pre-aggregated as needed.
            let xs = self.df.group_values(gi, &self.x_column);
            let ys = self.df.group_values(gi, &self.y_column);

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

        Axis { opts, elements }
    }
}
