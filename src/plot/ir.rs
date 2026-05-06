use crate::plot::fmt_f;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PlotDocument {
    pub(crate) setup_lines: Vec<String>,
    pub(crate) axes: Vec<Axis>,
}

impl PlotDocument {
    pub(crate) fn render_tikz(&self) -> String {
        let mut out = String::new();

        if !self.setup_lines.is_empty() {
            for (idx, line) in self.setup_lines.iter().enumerate() {
                if idx > 0 {
                    out.push('\n');
                }
                out.push_str(line);
            }
            out.push_str("\n\n");
        }

        out.push_str("\\begin{tikzpicture}\n");

        for axis in &self.axes {
            out.push_str(&axis.render_tikz());
        }

        out.push_str("\\end{tikzpicture}\n");
        out
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Axis {
    pub(crate) options: Vec<AxisOption>,
    pub(crate) elements: Vec<AxisElement>,
}

impl Axis {
    fn render_tikz(&self) -> String {
        let mut out = String::from("\\begin{axis}[\n");
        for option in &self.options {
            out.push_str("  ");
            out.push_str(&option.render_tikz());
            out.push_str(",\n");
        }
        out.push_str("]\n");

        for element in &self.elements {
            out.push_str(&element.render_tikz());
        }

        out.push_str("\\end{axis}\n");
        out
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AxisOption {
    Flag(String),
    KeyValue { key: String, value: String },
}

impl AxisOption {
    pub(crate) fn flag(value: impl Into<String>) -> Self {
        Self::Flag(value.into())
    }

    pub(crate) fn key_value(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self::KeyValue { key: key.into(), value: value.into() }
    }

    fn render_tikz(&self) -> String {
        match self {
            Self::Flag(value) => value.clone(),
            Self::KeyValue { key, value } => format!("{key}={value}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AxisElement {
    Plot(AddPlot),
    LegendEntry(String),
    LegendImage(Vec<String>),
    DrawLine { options: Vec<String>, from: Coordinate, to: Coordinate },
}

impl AxisElement {
    fn render_tikz(&self) -> String {
        match self {
            Self::Plot(plot) => plot.render_tikz(),
            Self::LegendEntry(label) => format!("\\addlegendentry{{{label}}}\n"),
            Self::LegendImage(options) => {
                format!("\\addlegendimage{{{}}}\n", options.join(", "))
            }
            Self::DrawLine { options, from, to } => format!(
                "\\draw[{}] {} -- {};\n",
                options.join(", "),
                from.render_tikz(),
                to.render_tikz()
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AddPlot {
    pub(crate) options: Vec<String>,
    pub(crate) coordinates: Vec<Coordinate>,
    pub(crate) closed_cycle: bool,
}

impl AddPlot {
    fn render_tikz(&self) -> String {
        let mut out = format!("\\addplot[{}]\n  coordinates {{\n", self.options.join(", "));
        for coordinate in &self.coordinates {
            out.push_str("    ");
            out.push_str(&coordinate.render_tikz());
            out.push('\n');
        }
        out.push_str("  }");
        if self.closed_cycle {
            out.push_str(" \\closedcycle;\n");
        } else {
            out.push_str(";\n");
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Coordinate {
    Plain(f64, f64),
    AxisCs(f64, f64),
}

impl Coordinate {
    fn render_tikz(&self) -> String {
        match self {
            Self::Plain(x, y) => format!("({}, {})", fmt_f(*x), fmt_f(*y)),
            Self::AxisCs(x, y) => format!("(axis cs:{},{})", fmt_f(*x), fmt_f(*y)),
        }
    }
}