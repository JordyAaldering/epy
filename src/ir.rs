use crate::color::Color;

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
    DrawLabel { options: Vec<String>, at: Coordinate, label: String },
}

#[derive(Clone, Debug)]
pub struct AddPlot {
    pub opts: Vec<String>,
    pub coords: Vec<Coordinate>,
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

    pub fn annot_area(mut self, xmin: f64, ymin: f64, xmax: f64, ymax: f64, color: Color) -> Self {
        let options = vec![
            format!("draw={}", color.tikz_name()),
            "draw opacity=0.5".into(),
            format!("postaction={{pattern=north east lines, pattern color={}, fill opacity=0.5}}", color.tikz_name()),
        ];
        let bottom_left = Coordinate::AxisCs(xmin, ymin);
        let top_right = Coordinate::AxisCs(xmax, ymax);
        self.ax0.elements.push(AxisElement::DrawArea { options, bottom_left, top_right });
        self
    }

    pub fn annot_label(mut self, x: f64, y: f64, label: &str) -> Self {
        let options = vec![
            "font=\\epyannotsize".into(),
        ];
        let at = Coordinate::AxisCs(x, y);
        self.ax0.elements.push(AxisElement::DrawLabel { options, at, label: label.into() });
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
        use AxisOption::*;
        match self {
            Flag(value) => value.clone(),
            KeyValue { key, value } => format!("{key}={value}"),
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
            DrawLabel { options, at, label } => {
                format!("\\draw {} node[{}] {{{}}};\n",
                    at.render_tikz(), options.join(","), label)
            }
        }
    }
}

impl AddPlot {
    pub fn render_tikz(&self) -> String {
        let mut out = format!("\\addplot[{}]\n  coordinates {{\n", self.opts.join(","));
        for coordinate in &self.coords {
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
        use Coordinate::*;
        match self {
            Plain(x, y) => format!("({},{})", x, y),
            AxisCs(x, y) => format!("(axis cs:{},{})", x, y),
        }
    }
}
