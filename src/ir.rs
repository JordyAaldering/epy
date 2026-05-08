use crate::color::Color;
use std::{collections::HashSet, hash::{Hash, Hasher}, mem};

#[derive(Clone, Debug)]
pub struct PlotDocument {
    pub setup_lines: Vec<String>,
    pub ax0: Axis,
    pub ax1: Option<Axis>,
}

#[derive(Clone, Debug)]
pub struct Axis {
    pub opts: HashSet<AxisOption>,
    pub elements: Vec<AxisElement>,
}

#[derive(Clone, Copy, Debug)]
pub struct Numeric(pub f64);

impl Numeric {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    fn render_tikz(self) -> String {
        self.0.to_string()
    }
}

impl PartialEq for Numeric {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for Numeric {}

impl Hash for Numeric {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Style {
    pub color: Option<Color>,
    pub draw: Option<Color>,
    pub line_width_pt: Option<Numeric>,
    pub font: Option<String>,
    pub inner_sep_pt: Option<Numeric>,
    pub fill_opacity: Option<Numeric>,
    pub draw_opacity: Option<Numeric>,
    pub text_opacity: Option<Numeric>,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_draw(mut self, color: Color) -> Self {
        self.draw = Some(color);
        self
    }

    pub fn with_line_width_pt(mut self, value: f64) -> Self {
        self.line_width_pt = Some(Numeric::new(value));
        self
    }

    pub fn with_font(mut self, font: impl Into<String>) -> Self {
        self.font = Some(font.into());
        self
    }

    pub fn with_inner_sep_pt(mut self, value: f64) -> Self {
        self.inner_sep_pt = Some(Numeric::new(value));
        self
    }

    pub fn with_fill_opacity(mut self, value: f64) -> Self {
        self.fill_opacity = Some(Numeric::new(value));
        self
    }

    pub fn with_draw_opacity(mut self, value: f64) -> Self {
        self.draw_opacity = Some(Numeric::new(value));
        self
    }

    pub fn with_text_opacity(mut self, value: f64) -> Self {
        self.text_opacity = Some(Numeric::new(value));
        self
    }

    fn render_tikz(&self) -> String {
        let mut parts = Vec::new();
        if let Some(color) = self.color {
            parts.push(format!("color={}", color.tikz_name()));
        }
        if let Some(draw) = self.draw {
            parts.push(format!("draw={}", draw.tikz_name()));
        }
        if let Some(line_width_pt) = self.line_width_pt {
            parts.push(format!("line width={}pt", line_width_pt.render_tikz()));
        }
        if let Some(font) = &self.font {
            parts.push(format!("font={font}"));
        }
        if let Some(inner_sep_pt) = self.inner_sep_pt {
            parts.push(format!("inner sep={}pt", inner_sep_pt.render_tikz()));
        }
        if let Some(fill_opacity) = self.fill_opacity {
            parts.push(format!("fill opacity={}", fill_opacity.render_tikz()));
        }
        if let Some(draw_opacity) = self.draw_opacity {
            parts.push(format!("draw opacity={}", draw_opacity.render_tikz()));
        }
        if let Some(text_opacity) = self.text_opacity {
            parts.push(format!("text opacity={}", text_opacity.render_tikz()));
        }
        parts.join(",")
    }
}

#[derive(Clone, Debug)]
pub enum AxisOption {
    ScaleOnlyAxis,
    AxisLineColor(Color),
    XGridColor(Color),
    YGridStyle(Style),
    TickAlignOutside,
    XTickPosLeft,
    YTickPosLeft,
    YTickPosRight,
    XMajorGrids(bool),
    YMajorGrids(bool),
    MajorTickLength(Numeric),
    XTickStyle(Style),
    YTickStyle(Style),
    TickLabelStyle(Style),
    LegendStyle(Style),
    EnsureAxisHeightExtraYTick,
    EnsureAxisHeightExtraYTickLabels,
    EnsureAxisHeightExtraYTickStyle,
    Name(String),
    TrimAxisRight,
    TrimAxisLeft,
    Width(String),
    Height(String),
    XLabel(String),
    YLabel(String),
    YMin(Numeric),
    YMax(Numeric),
    XMin(Numeric),
    XMax(Numeric),
    XTicks(Vec<String>),
    EmptyXTicks,
    XTickLabels(Vec<String>),
    EmptyXTickLabels,
    AtMainAxisSouthWest,
    AnchorSouthWest,
    AxisXLineNone,
    AxisYLineRight,
    ScaledTicksFalse,
    TickNumberFormatFixed,
}

impl PartialEq for AxisOption {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl Eq for AxisOption {}

impl Hash for AxisOption {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
    }
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
    /// Set the minimum y-axis value.
    pub fn set_ymin(&mut self, value: f64) {
        self.opts.replace(AxisOption::YMin(Numeric::new(value)));
    }

    /// Set the maximum y-axis value.
    pub fn set_ymax(&mut self, value: f64) {
        self.opts.replace(AxisOption::YMax(Numeric::new(value)));
    }

    /// Filter x-ticks and x-tick labels to show every nth tick (stride >= 1).
    /// Keeps every stride-th element; stride=1 keeps all, stride=2 keeps every other, etc.
    pub fn filter_xticks_stride(&mut self, stride: usize) {
        let mut filtered_ticks = Vec::new();
        let mut filtered_labels = Vec::new();

        // Extract current ticks and labels from opts
        let mut current_ticks: Option<Vec<String>> = None;
        let mut current_labels: Option<Vec<String>> = None;

        for opt in &self.opts {
            if let AxisOption::XTicks(ticks) = opt {
                current_ticks = Some(ticks.clone());
            }
            if let AxisOption::XTickLabels(labels) = opt {
                current_labels = Some(labels.clone());
            }
        }

        // Filter ticks: keep every stride-th
        if let Some(ticks) = current_ticks {
            for (i, tick) in ticks.iter().enumerate() {
                if i % stride == 0 {
                    filtered_ticks.push(tick.clone());
                }
            }
            if !filtered_ticks.is_empty() {
                self.opts.replace(AxisOption::XTicks(filtered_ticks));
            }
        }

        // Filter labels to match kept ticks
        if let Some(labels) = current_labels {
            for (i, label) in labels.iter().enumerate() {
                if i % stride == 0 {
                    filtered_labels.push(label.clone());
                }
            }
            if !filtered_labels.is_empty() {
                self.opts.replace(AxisOption::XTickLabels(filtered_labels));
            }
        }
    }

    fn render_tikz(&self) -> String {
        let mut out = String::from("\\begin{axis}[\n");
        for opt in &self.opts {
            out.push_str("  ");
            out.push_str(&opt.render_tikz());
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
    fn render_tikz(&self) -> String {
        match self {
            AxisOption::ScaleOnlyAxis => "scale only axis".into(),
            AxisOption::AxisLineColor(color) => format!("axis line style={{{}}}", color.tikz_name()),
            AxisOption::XGridColor(color) => format!("x grid style={{{}}}", color.tikz_name()),
            AxisOption::YGridStyle(style) => format!("y grid style={{{}}}", style.render_tikz()),
            AxisOption::TickAlignOutside => "tick align=outside".into(),
            AxisOption::XTickPosLeft => "xtick pos=left".into(),
            AxisOption::YTickPosLeft => "ytick pos=left".into(),
            AxisOption::YTickPosRight => "ytick pos=right".into(),
            AxisOption::XMajorGrids(true) => "xmajorgrids".into(),
            AxisOption::XMajorGrids(false) => "xmajorgrids=false".into(),
            AxisOption::YMajorGrids(true) => "ymajorgrids".into(),
            AxisOption::YMajorGrids(false) => "ymajorgrids=false".into(),
            AxisOption::MajorTickLength(value) => format!("major tick length={}pt", value.render_tikz()),
            AxisOption::XTickStyle(style) => format!("xtick style={{{}}}", style.render_tikz()),
            AxisOption::YTickStyle(style) => format!("ytick style={{{}}}", style.render_tikz()),
            AxisOption::TickLabelStyle(style) => format!("tick label style={{{}}}", style.render_tikz()),
            AxisOption::LegendStyle(style) => format!("legend style={{{}}}", style.render_tikz()),
            AxisOption::EnsureAxisHeightExtraYTick => "extra y ticks={\\pgfkeysvalueof{/pgfplots/ymax}}".into(),
            AxisOption::EnsureAxisHeightExtraYTickLabels => "extra y tick labels={\\vphantom{Ag}}".into(),
            AxisOption::EnsureAxisHeightExtraYTickStyle => "extra y tick style={yticklabel style={opacity=0,text opacity=0},major tick length=0pt,grid=none}".into(),
            AxisOption::Name(name) => format!("name={name}"),
            AxisOption::TrimAxisRight => "trim axis right".into(),
            AxisOption::TrimAxisLeft => "trim axis left".into(),
            AxisOption::Width(width) => format!("width={width}"),
            AxisOption::Height(height) => format!("height={height}"),
            AxisOption::XLabel(label) => format!("xlabel={{\\epylabelsize {label}}}"),
            AxisOption::YLabel(label) => format!("ylabel={{\\epylabelsize {label}}}"),
            AxisOption::YMin(value) => format!("ymin={}", value.render_tikz()),
            AxisOption::YMax(value) => format!("ymax={}", value.render_tikz()),
            AxisOption::XMin(value) => format!("xmin={}", value.render_tikz()),
            AxisOption::XMax(value) => format!("xmax={}", value.render_tikz()),
            AxisOption::XTicks(values) => format!("xtick={{{}}}", values.join(",")),
            AxisOption::EmptyXTicks => "xtick=\\empty".into(),
            AxisOption::XTickLabels(values) => format!("xticklabels={{{}}}", values.join(",")),
            AxisOption::EmptyXTickLabels => "xticklabels=\\empty".into(),
            AxisOption::AtMainAxisSouthWest => "at={(mainaxis.south west)}".into(),
            AxisOption::AnchorSouthWest => "anchor=south west".into(),
            AxisOption::AxisXLineNone => "axis x line=none".into(),
            AxisOption::AxisYLineRight => "axis y line=right".into(),
            AxisOption::ScaledTicksFalse => "scaled ticks=false".into(),
            AxisOption::TickNumberFormatFixed => "/pgf/number format/fixed".into(),
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
