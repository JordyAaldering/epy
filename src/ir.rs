#[derive(Clone, Debug)]
pub struct PlotDocument {
    setup_lines: Vec<String>,
    ax0: Axis,
    ax1: Option<Axis>,
}

#[derive(Clone, Debug)]
pub struct Axis {
    pub(crate) opts: Vec<AxisOption>,
    pub(crate) elements: Vec<AxisElement>,
}

#[derive(Clone, Debug)]
pub enum AxisOption {
    Flag(String),
    KeyValue { key: String, value: String },
}

#[derive(Clone, Debug)]
pub enum AxisElement {
    Plot(AddPlot),
    LegendEntry(String),
    LegendImage(Vec<String>),
    DrawLine { options: Vec<String>, from: Coordinate, to: Coordinate },
    DrawArea { options: Vec<String>, bottom_left: Coordinate, top_right: Coordinate },
}

#[derive(Clone, Debug)]
pub struct AddPlot {
    pub options: Vec<String>,
    pub coordinates: Vec<Coordinate>,
    pub closed_cycle: bool,
}

#[derive(Clone, Debug)]
pub enum Coordinate {
    Plain(f64, f64),
    AxisCs(f64, f64),
}

impl PlotDocument {
    pub fn new(setup_lines: Vec<String>, ax0: Axis, ax1: Option<Axis>) -> Self {
        PlotDocument { setup_lines, ax0, ax1 }
    }

    pub fn annot_area(mut self, xmin: f64, xmax: f64, ymin: f64, ymax: f64) -> Self {
        let options = vec![
            "draw=epyannotcolor".into(),
            "draw opacity=0.5".into(),
            "postaction={pattern=north east lines, pattern color=epyannotcolor, fill opacity=0.5}".into(),
        ];
        let bottom_left = Coordinate::AxisCs(xmin, ymin);
        let top_right = Coordinate::AxisCs(xmax, ymax);
        self.ax0.elements.push(AxisElement::DrawArea { options, bottom_left, top_right });
        self
    }

    pub fn render_tikz(&self) -> String {
        let mut out = String::new();
        for line in &self.setup_lines {
            out.push_str(line);
            out.push('\n');
        }
        out.push_str("\\begin{tikzpicture}\n");
        out.push_str(&self.ax0.render_tikz());
        if let Some(ax1) = &self.ax1 {
            out.push_str(&ax1.render_tikz());
        }
        out.push_str("\\end{tikzpicture}\n");
        out
    }
}

impl Axis {
    fn render_tikz(&self) -> String {
        let mut out = String::from("\\begin{axis}[\n");
        for option in &self.opts {
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

impl AxisOption {
    pub fn flag(value: impl Into<String>) -> Self {
        Self::Flag(value.into())
    }

    pub fn key_value(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self::KeyValue { key: key.into(), value: value.into() }
    }

    fn render_tikz(&self) -> String {
        match self {
            Self::Flag(value) => value.clone(),
            Self::KeyValue { key, value } => format!("{key}={value}"),
        }
    }
}

impl AxisElement {
    pub fn render_tikz(&self) -> String {
        use AxisElement::*;
        match self {
            Plot(plot) => {
                plot.render_tikz()
            }
            LegendEntry(label) => {
                format!("\\addlegendentry{{{label}}}\n")
            }
            LegendImage(options) => {
                format!("\\addlegendimage{{{}}}\n", options.join(","))
            }
            DrawLine { options, from, to } => {
                format!("\\draw[{}] {} -- {};\n",
                    options.join(","), from.render_tikz(), to.render_tikz())
            }
            DrawArea { options, bottom_left, top_right } => {
                format!("\\draw[{}] {} rectangle {};\n",
                    options.join(","), bottom_left.render_tikz(), top_right.render_tikz())
            }
        }
    }
}

impl AddPlot {
    pub fn render_tikz(&self) -> String {
        let mut out = format!("\\addplot[{}]\n  coordinates {{\n", self.options.join(","));
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

impl Coordinate {
    pub fn render_tikz(&self) -> String {
        match self {
            Self::Plain(x, y) => format!("({}, {})", x, y),
            Self::AxisCs(x, y) => format!("(axis cs:{},{})", x, y),
        }
    }
}
