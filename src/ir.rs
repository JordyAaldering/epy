use std::{collections::HashSet, hash::{Hash, Hasher}, mem};

pub(crate) const MAJOR_TICK_LENGTH_EM: f64 = 0.3;

/// Outer sep affects only the distance between the tick values and the axis.
pub(crate) const TICK_LABEL_OUTER_SEP_EM: f64 = -0.25;
/// Inner sep affects both he distance between the tick values and the axis,
/// and the distance between the tick values and the axis label.
pub(crate) const TICK_LABEL_INNER_SEP_EM: f64 = 0.15;
/// Extra right padding for twin-axis plots.
pub(crate) const TWIN_PADDING_EM: f64 = 1.0;

pub(crate) const GRID_COLOR: &'static str = "black!20";

/// See: https://tikz.dev/pgfplots/reference-markers
pub(crate) const MARKERS: &[&str] = &[
    "*", "square*", "pentagon*", "diamond*", "triangle*",
    "halfcircle*", "halfsquare*", "halfdiamond*"];
pub(crate) const MARK_SIZE_PT: f64 = 2.0;
pub(crate) const MARK_OUTLINE_PT: f64 = MARK_SIZE_PT / 5.0;

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
    pub color: Option<String>,
    pub draw: Option<String>,
    pub line_width_pt: Option<Numeric>,
    pub inner_sep_em: Option<Numeric>,
    pub outer_sep_em: Option<Numeric>,
    pub fill_opacity: Option<Numeric>,
    pub draw_opacity: Option<Numeric>,
    pub text_opacity: Option<Numeric>,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_draw(mut self, color: String) -> Self {
        self.draw = Some(color);
        self
    }

    pub fn with_line_width_pt(mut self, value: f64) -> Self {
        self.line_width_pt = Some(Numeric::new(value));
        self
    }

    pub fn with_inner_sep_em(mut self, value: f64) -> Self {
        self.inner_sep_em = Some(Numeric::new(value));
        self
    }

    pub fn with_outer_sep_em(mut self, value: f64) -> Self {
        self.outer_sep_em = Some(Numeric::new(value));
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
        if let Some(color) = &self.color {
            parts.push(format!("color={}", color));
        }
        if let Some(draw) = &self.draw {
            parts.push(format!("draw={}", draw));
        }
        if let Some(line_width_pt) = self.line_width_pt {
            parts.push(format!("line width={}pt", line_width_pt.render_tikz()));
        }
        if let Some(inner_sep_em) = self.inner_sep_em {
            parts.push(format!("inner sep={}em", inner_sep_em.render_tikz()));
        }
        if let Some(outer_sep_em) = self.outer_sep_em {
            parts.push(format!("outer sep={}em", outer_sep_em.render_tikz()));
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
    AxisLineColor(String),
    SetXGridColor,
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
    YTickLabelStyle(Style),
    LegendStyle(Style),
    LegendCellAlignLeft,
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
    YLabelStyle(Style),
    YMin(Numeric),
    YMax(Numeric),
    XMin(Numeric),
    XMax(Numeric),
    XTicks(Vec<String>),
    EmptyXTicks,
    XTickLabels(Vec<String>),
    EmptyXTickLabels,
    XTickDistance(Numeric),
    AtMainAxisSouthWest,
    AnchorSouthWest,
    AxisXLineNone,
    AxisYLineRight,
    ScaledTicksFalse,
    TickNumberFormatFixed,
    LegendPos(String),
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

    pub fn from_axis(ax0: Axis) -> Self {
        Self::new(Self::single_setup_lines(), ax0, None)
    }

    pub fn from_twin_axes(ax0: Axis, ax1: Axis) -> Self {
        let setup_lines = Self::twin_setup_lines(&ax1);
        Self::new(setup_lines, ax0, Some(ax1))
    }

    fn single_setup_lines() -> Vec<String> {
        Vec::new()
    }

    fn twin_setup_lines(right_axis: &Axis) -> Vec<String> {
        let tick_estimate = right_axis
            .opts
            .iter()
            .find_map(|opt| {
                if let AxisOption::YMax(value) = opt {
                    Some(value.0.to_string())
                } else {
                    None
                }
            })
            // Assume a default tick precision if no upper bound is found
            .unwrap_or("0.00".to_string());

        let mut lines = Vec::new();
        lines.push("\\ifx\\epyrpad\\undefined\\newlength{\\epyrpad}\\fi%".into());
        lines.push(format!("\\settowidth{{\\epyrpad}}{{\\normalfont {tick_estimate}}}%"));
        // Representative glyph sample used to estimate tick-label ascent/height when
        // reserving right-axis padding. "Ag" provides a stable height across fonts.
        lines.push("\\begingroup\\settoheight{\\dimen0}{\\normalfont Ag}\\addtolength{\\epyrpad}{\\dimen0}\\endgroup%".into());
        // An axis label is always assumed to present.
        lines.push(format!("\\addtolength{{\\epyrpad}}{{{}em}}%", TWIN_PADDING_EM));
        lines
    }

    pub fn annot_label(mut self, (x, y): (f64, f64), label: &str, anchor: Option<&str>) -> Self {
        let options = vec![
            format!("anchor={}", anchor.unwrap_or("south west")),
            "inner xsep=0em".into(),
            "inner ysep=0.1em".into(),
        ];
        let at = Coordinate::AxisCs(x, y);
        self.ax0.elements.push(AxisElement::DrawLabel { options, at, label: label.into() });
        self
    }

    pub fn annot_line(mut self, (x1, y1): (f64, f64), (x2, y2): (f64, f64), color: &str) -> Self {
        let options = vec![
            format!("draw={}", color),
            "dashed".into(),
        ];
        let from = Coordinate::AxisCs(x1, y1);
        let to = Coordinate::AxisCs(x2, y2);
        self.ax0.elements.push(AxisElement::DrawLine { options, from, to });
        self
    }

    pub fn annot_area(mut self, (x1, y1): (f64, f64), (x2, y2): (f64, f64), color: &str) -> Self {
        let options = vec![
            format!("draw={}", color),
            "draw opacity=0.5".into(),
            format!("postaction={{pattern=north east lines, pattern color={}, fill opacity=0.5}}", color),
        ];
        let bottom_left = Coordinate::AxisCs(x1, y1);
        let top_right = Coordinate::AxisCs(x2, y2);
        self.ax0.elements.push(AxisElement::DrawArea { options, bottom_left, top_right });
        self
    }

    pub fn render_tikz(&self) -> String {
        let mut out = String::new();
        for line in &self.setup_lines {
            out.push_str(line);
            out.push('\n');
        }
        // Add bounding box for debugging
        //out.push_str("\\begin{tikzpicture}[show background rectangle]\n");
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
    pub fn set_xmin(mut self, value: f64) -> Self {
        self.opts.replace(AxisOption::XMin(Numeric::new(value)));
        self
    }

    pub fn set_xmax(mut self, value: f64) -> Self {
        self.opts.replace(AxisOption::XMax(Numeric::new(value)));
        self
    }

    pub fn set_ymin(mut self, value: f64) -> Self {
        self.opts.replace(AxisOption::YMin(Numeric::new(value)));
        self
    }

    pub fn set_ymax(mut self, value: f64) -> Self {
        self.opts.replace(AxisOption::YMax(Numeric::new(value)));
        self
    }

    pub fn set_xtick_distance(mut self, distance: f64) -> Self {
        self.opts.remove(&AxisOption::XTicks(Vec::new()));
        self.opts.remove(&AxisOption::EmptyXTicks);
        self.opts.remove(&AxisOption::XTickLabels(Vec::new()));
        self.opts.remove(&AxisOption::EmptyXTickLabels);
        self.opts.replace(AxisOption::XTickDistance(Numeric::new(distance)));
        self
    }

    pub fn set_legend_pos(mut self, position: impl Into<String>) -> Self {
        self.opts.replace(AxisOption::LegendPos(position.into()));
        self
    }

    pub fn filter_xticks_stride(mut self, start: usize, stride: usize) -> Self {
        let mut filtered_ticks = Vec::new();
        let mut filtered_labels = Vec::new();

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
                if i >= start && (i - start) % stride == 0 {
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
                if i >= start && (i - start) % stride == 0 {
                    filtered_labels.push(label.clone());
                }
            }
            if !filtered_labels.is_empty() {
                self.opts.replace(AxisOption::XTickLabels(filtered_labels));
            }
        }

        self
    }

    pub fn format_xticks_precision(self, precision: usize, trim: bool) -> Self {
        self.format_xticks(|s| {
            if trim {
                format!("{:.precision$}", s.parse::<f64>().unwrap())
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string()
            } else {
                format!("{:.precision$}", s.parse::<f64>().unwrap())
            }
        })
    }

    pub fn format_xticks(mut self, fmt: impl Fn(&str) -> String) -> Self {
        let mut formatted_ticks = Vec::new();
        let mut formatted_labels = Vec::new();

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

        // Format ticks
        if let Some(ticks) = current_ticks {
            for tick in ticks.iter() {
                formatted_ticks.push(fmt(tick));
            }
            if !formatted_ticks.is_empty() {
                self.opts.replace(AxisOption::XTicks(formatted_ticks));
            }
        }

        // Format labels to match formatted ticks
        if let Some(labels) = current_labels {
            for label in labels.iter() {
                formatted_labels.push(fmt(label));
            }
            if !formatted_labels.is_empty() {
                self.opts.replace(AxisOption::XTickLabels(formatted_labels));
            }
        }

        self
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
        use AxisOption::*;
        match self {
            ScaleOnlyAxis => "scale only axis".into(),
            AxisLineColor(color) => format!("axis line style={{{}}}", color),
            SetXGridColor => format!("x grid style={{{}}}", GRID_COLOR),
            YGridStyle(style) => format!("y grid style={{{}}}", style.render_tikz()),
            TickAlignOutside => "tick align=outside".into(),
            XTickPosLeft => "xtick pos=left".into(),
            YTickPosLeft => "ytick pos=left".into(),
            YTickPosRight => "ytick pos=right".into(),
            XMajorGrids(true) => "xmajorgrids".into(),
            XMajorGrids(false) => "xmajorgrids=false".into(),
            YMajorGrids(true) => "ymajorgrids".into(),
            YMajorGrids(false) => "ymajorgrids=false".into(),
            MajorTickLength(value) => format!("major tick length={}em", value.render_tikz()),
            XTickStyle(style) => format!("xtick style={{{}}}", style.render_tikz()),
            YTickStyle(style) => format!("ytick style={{{}}}", style.render_tikz()),
            TickLabelStyle(style) => format!("tick label style={{{}}}", style.render_tikz()),
            YTickLabelStyle(style) => format!("yticklabel style={{{}}}", style.render_tikz()),
            LegendStyle(style) => format!("legend style={{{}}}", style.render_tikz()),
            LegendCellAlignLeft => "legend cell align=left".into(),
            EnsureAxisHeightExtraYTick => "extra y ticks={\\pgfkeysvalueof{/pgfplots/ymax}}".into(),
            EnsureAxisHeightExtraYTickLabels => "extra y tick labels={\\vphantom{Ag}}".into(),
            EnsureAxisHeightExtraYTickStyle => "extra y tick style={yticklabel style={opacity=0,text opacity=0},major tick length=0pt,grid=none}".into(),
            Name(name) => format!("name={name}"),
            TrimAxisRight => "trim axis right".into(),
            TrimAxisLeft => "trim axis left".into(),
            Width(width) => format!("width={width}"),
            Height(height) => format!("height={height}"),
            XLabel(label) => format!("xlabel={{{label}}}"),
            YLabel(label) => format!("ylabel={{{label}}}"),
            YLabelStyle(style) => format!("ylabel style={{{}}}", style.render_tikz()),
            YMin(value) => format!("ymin={}", value.render_tikz()),
            YMax(value) => format!("ymax={}", value.render_tikz()),
            XMin(value) => format!("xmin={}", value.render_tikz()),
            XMax(value) => format!("xmax={}", value.render_tikz()),
            XTicks(values) => format!("xtick={{{}}}", values.join(",")),
            EmptyXTicks => "xtick=\\empty".into(),
            XTickLabels(values) => format!("xticklabels={{{}}}", values.join(",")),
            EmptyXTickLabels => "xticklabels=\\empty".into(),
            XTickDistance(value) => format!("xtick distance={}", value.render_tikz()),
            AtMainAxisSouthWest => "at={(mainaxis.south west)}".into(),
            AnchorSouthWest => "anchor=south west".into(),
            AxisXLineNone => "axis x line=none".into(),
            AxisYLineRight => "axis y line*=right".into(),
            ScaledTicksFalse => "scaled ticks=false".into(),
            TickNumberFormatFixed => "/pgf/number format/fixed".into(),
            LegendPos(pos) => format!("legend pos={pos}"),
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
                if options.is_empty() {
                    format!("\\draw {} node {{{}}};\n", at.render_tikz(), label)
                } else {
                    format!("\\draw {} node[{}] {{{}}};\n",
                        at.render_tikz(), options.join(","), label)
                }
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
