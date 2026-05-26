//! TikZ intermediate representation
use derive_builder::Builder;
use ordermap::OrderMap;

#[derive(Clone, Debug)]
pub struct TikzPicture {
    pub ax0: Axis,
    pub ax1: Option<Axis>,
}

impl TikzPicture {
    pub fn from_axis(ax0: Axis) -> Self {
        Self { ax0, ax1: None }
    }

    pub fn from_twin(ax0: Axis, ax1: Axis) -> Self {
        Self { ax0, ax1: Some(ax1) }
    }

    pub fn render(&self) -> String {
        let mut res = String::new();
        if let Some(ax1) = &self.ax1 {
            res.push_str(&self.size_calculation(ax1));
        }
        // Add bounding box for debugging
        //out.push_str("\\begin{tikzpicture}[show background rectangle]\n");
        res.push_str("\\begin{tikzpicture}\n");
        res.push_str(&self.ax0.render());
        res.push('\n');
        if let Some(ax1) = &self.ax1 {
            res.push_str(&ax1.render());
            res.push('\n');
        }
        res.push_str("\\end{tikzpicture}");
        res
    }

    fn size_calculation(&self, ax1: &Axis) -> String {
        let tick_estimate = ax1.style.ymax
            .map_or("0.00".to_string(), |v| v.to_string());
        let mut res = String::new();
        res.push_str("\\ifx\\epyrpad\\undefined\\newlength{\\epyrpad}\\fi%\n".into());
        res.push_str(&format!("\\settowidth{{\\epyrpad}}{{\\normalfont {tick_estimate}}}%\n"));
        // Representative glyph sample used to estimate tick-label ascent/height when
        // reserving right-axis padding. "Ag" provides a stable height across fonts.
        res.push_str("\\begingroup\\settoheight{\\dimen0}{\\normalfont Ag}\\addtolength{\\epyrpad}{\\dimen0}\\endgroup%\n".into());
        // An axis label is always assumed to present.
        res.push_str("\\addtolength{\\epyrpad}{1em}%\n".into());
        res
    }
}

#[derive(Clone, Debug)]
pub struct Axis {
    pub style: Style,
    pub data: Vec<AxisElement>,
}

impl Axis {
    pub fn render(&self) -> String {
        let mut res = String::new();
        res.push_str("\\begin{axis}");

        let options = self.style.render();
        if !options.is_empty() {
            res.push('[');
            res.push_str(&options.into_iter()
                .map(|s| format!("\n  {},", s))
                .collect::<String>());
            res.push('\n');
            res.push(']');
        }
        res.push('\n');

        for element in &self.data {
            res.push_str(&element.render());
            res.push('\n');
        }

        res.push_str("\\end{axis}");
        res
    }
}

#[derive(Clone, Debug)]
pub enum AxisElement {
    Draw {
        options: Option<Style>,
        from: Coordinate,
        to: Coordinate,
    },
    AddPlot {
        options: Option<Style>,
        coordinates: Vec<Coordinate>,
        closed_cycle: bool,
    },
    LegendEntry(String),
    LegendImage(Style),
}

impl AxisElement {
    pub fn render(&self) -> String {
        use AxisElement::*;
        match self {
            Draw { options, from, to } => {
                if let Some(options) = options {
                    format!("\\draw[{}] {} -- {};", options.render().join(","), from.render(), to.render())
                } else {
                    format!("\\draw {} -- {};", from.render(), to.render())
                }
            }
            AddPlot { options, coordinates, closed_cycle } => {
                let mut res = String::new();
                res.push_str("\\addplot");
                if let Some(options) = options {
                    let options = options.render();
                    if !options.is_empty() {
                        res.push('[');
                        res.push_str(&options.join(","));
                        res.push(']');
                    }
                }
                res.push_str(" coordinates {");
                for c in coordinates {
                    res.push_str("\n  ");
                    res.push_str(&c.render());
                }
                res.push('\n');
                res.push('}');
                if *closed_cycle {
                    res.push_str(" \\closedcycle");
                }
                res.push(';');
                res
            }
            LegendEntry(label) => format!("\\addlegendentry{{{}}}", label),
            LegendImage(style) => format!("\\addlegendimage{{{}}}", style.render().join(",")),
        }
    }
}

#[derive(Builder, Clone, Debug, Default)]
#[builder(default, setter(into, strip_option))]
pub struct Style {
    pub name: Option<String>,

    pub width: Option<Dimension>,
    pub height: Option<Dimension>,

    pub anchor: Option<Anchor>,
    pub at: Option<String>,

    pub ybar: bool,
    pub bar_width: Option<f64>,

    pub xmin: Option<f64>,
    pub xmax: Option<f64>,
    pub ymin: Option<f64>,
    pub ymax: Option<f64>,

    pub xlabel: Option<String>,
    pub ylabel: Option<String>,

    /// These options assign a list of Positions where ticks shall be placed.
    /// The argument is either the empty string (which is the initial value),
    /// the command \empty, data or a list of coordinates. The initial
    /// configuration of an empty string means to generate these positions
    /// automatically. The choice \empty will result in no tick at all.
    /// The special value data will produce tick marks at every coordinate of
    /// the first plot. Otherwise, tick marks will be placed at every
    /// coordinate in {〈coordinate list〉}.
    #[builder(default = TickPositions::Auto)]
    pub xticks: TickPositions,
    #[builder(default = TickPositions::Auto)]
    pub yticks: TickPositions,

    /// Allows to factor out common exponents in tick labels for linear axes.
    ///
    /// Actually has more values than true/false, but I omit those for now:
    /// `true | false | base 10:〈e〉 | real:〈num〉 | manual:{〈label〉}{〈code〉}`
    ///
    /// Default: true
    #[builder(default = true)]
    pub scaled_ticks: bool,

    /// Adds additional tick positions and tick labels to the x or y axis.
    /// ‘Additional’ tick positions do not affect the normal tick placement
    /// algorithms, they are drawn after the normal ticks. This has two
    /// benefits: first, you can add single, important tick positions without
    /// disabling the default tick label generation and second, you can draw
    /// tick labels ‘on top’ of others, possibly using different style flags.
    pub extra_xticks: Option<Coordinates>,
    pub extra_yticks: Option<Coordinates>,

    /// Assigns a list of tick labels to each tick position.
    /// Tick positions are assigned using the xtick and ytick-options.
    pub xtick_labels: Option<TickLabels>,
    pub ytick_labels: Option<TickLabels>,
    /// As xticklabels provides explicit tick labels for each xtick,
    /// the key extra x tick labels provides explicit tick labels
    /// for every element in extra x ticks.
    pub extra_xtick_labels: Option<TickLabels>,
    pub extra_ytick_labels: Option<TickLabels>,

    /// Allows to choose where to place the small tick lines. In the default
    /// configuration, this does also affect tick labels.
    pub xtick_pos: Option<TickPos>,
    pub ytick_pos: Option<TickPos>,

    /// Allows to change the location of the ticks relative to the axis lines.
    pub tick_align: Option<TickAlign>,
    pub xtick_align: Option<TickAlign>,
    pub ytick_align: Option<TickAlign>,

    /// The distance between generated tick positions.
    pub xtick_distance: Option<f64>,
    pub ytick_distance: Option<f64>,

    pub x_major_ticks: Option<bool>,
    pub y_major_ticks: Option<bool>,
    pub x_minor_ticks: Option<bool>,
    pub y_minor_ticks: Option<bool>,
    pub ticks: Option<GridLines>,

    /// Enables/disables different grid lines. Major grid lines are placed at
    /// the normal tick positions (see [`x_major_ticks`]) while minor grid
    /// lines are placed at minor ticks (see [`x_minor_ticks`]`).
    pub x_major_grids: bool,
    pub y_major_grids: bool,
    pub x_minor_grids: bool,
    pub y_minor_grids: bool,
    pub grid: Option<GridLines>,

    pub legend_pos: Option<Anchor>,
    pub legend_cell_align: Option<CellAlign>,
    pub area_legend: bool,

    pub trim_axis_left: bool,
    pub trim_axis_right: bool,
    /// The starred versions ...line* only affect the axis lines, without correcting the positions of axis labels,
    /// tick lines or other keys which are (possibly) affected by a changed axis line. The non-starred versions
    /// are actually styles which set the starred key and some other keys which also affect the figure layout
    pub axis_x_line_star: Option<CellAlign>,
    pub axis_x_line: Option<CellAlign>,
    pub axis_y_line_star: Option<CellAlign>,
    pub axis_y_line: Option<CellAlign>,

    pub forget_plot: bool,

    // Styling

    pub color: Option<String>,
    pub draw: Option<String>,
    pub fill: Option<String>,

    pub opacity: Option<f64>,
    pub draw_opacity: Option<f64>,
    pub fill_opacity: Option<f64>,
    pub text_opacity: Option<f64>,

    pub line_width: Option<Dimension>,

    pub solid: bool,
    pub mark: Option<String>,
    pub mark_size: Option<Dimension>,
    pub mark_options: Option<Box<Style>>,

    pub major_tick_length: Option<Dimension>,
    pub minor_tick_length: Option<Dimension>,

    pub inner_sep: Option<Dimension>,
    pub inner_xsep: Option<Dimension>,
    pub inner_ysep: Option<Dimension>,
    pub outer_sep: Option<Dimension>,
    pub outer_xsep: Option<Dimension>,
    pub outer_ysep: Option<Dimension>,

    pub number_format: Option<NumberFormat>,

    pub style_overrides: OrderMap<String, Style>,
}

impl Style {
    pub fn render(&self) -> Vec<String> {
        let mut options = Vec::new();

        if let Some(name) = &self.name {
            options.push(format!("name={}", name));
        }

        if let Some(width) = &self.width {
            options.push(format!("width={}", width.render()));
        }
        if let Some(height) = &self.height {
            options.push(format!("height={}", height.render()));
        }

        if let Some(anchor) = &self.anchor {
            options.push(format!("anchor={}", anchor.render()));
        }
        if let Some(at) = &self.at {
            options.push(format!("at={{{}}}", at));
        }

        if self.ybar {
            options.push("ybar".into());
        }
        if let Some(bar_width) = self.bar_width {
            options.push(format!("bar width={}", bar_width));

        }

        if let Some(xmin) = self.xmin {
            options.push(format!("xmin={}", xmin));
        }
        if let Some(xmax) = self.xmax {
            options.push(format!("xmax={}", xmax));
        }
        if let Some(ymin) = self.ymin {
            options.push(format!("ymin={}", ymin));
        }
        if let Some(ymax) = self.ymax {
            options.push(format!("ymax={}", ymax));
        }

        if let Some(xlabel) = &self.xlabel {
            options.push(format!("xlabel={{{}}}", xlabel));
        }
        if let Some(ylabel) = &self.ylabel {
            options.push(format!("ylabel={{{}}}", ylabel));
        }

        if !matches!(self.xticks, TickPositions::Auto) {
            options.push(format!("xtick={}", self.xticks.render()));
        }
        if !matches!(self.yticks, TickPositions::Auto) {
            options.push(format!("ytick={}", self.yticks.render()));
        }

        if let Some(c) = &self.extra_xticks {
            options.push(format!("extra x ticks={}", c.render()));
        }
        if let Some(c) = &self.extra_yticks {
            options.push(format!("extra y ticks={}", c.render()));
        }

        if let Some(l) = &self.xtick_labels {
            options.push(format!("xticklabels={{{}}}", l.render()));
        }
        if let Some(l) = &self.ytick_labels {
            options.push(format!("yticklabels={{{}}}", l.render()));
        }
        if let Some(l) = &self.extra_xtick_labels {
            options.push(format!("extra x tick labels={{{}}}", l.render()));
        }
        if let Some(l) = &self.extra_ytick_labels {
            options.push(format!("extra y tick labels={{{}}}", l.render()));
        }

        if let Some(p) = &self.xtick_pos {
            options.push(format!("xtick pos={}", p.render()));
        }
        if let Some(p) = &self.ytick_pos {
            options.push(format!("ytick pos={}", p.render()));
        }

        if let Some(a) = &self.tick_align {
            options.push(format!("tick align={}", a.render()));
        }
        if let Some(a) = &self.xtick_align {
            options.push(format!("xtick align={}", a.render()));
        }
        if let Some(a) = &self.ytick_align {
            options.push(format!("ytick align={}", a.render()));
        }

        if let Some(d) = self.xtick_distance {
            options.push(format!("xtick distance={}", d));
        }
        if let Some(d) = self.ytick_distance {
            options.push(format!("ytick distance={}", d));
        }

        if let Some(x) = self.x_major_ticks {
            options.push(format!("xmajorticks={}", if x { "true" } else { "false" }));
        }
        if let Some(x) = self.y_major_ticks {
            options.push(format!("ymajorticks={}", if x { "true" } else { "false" }));
        }
        if let Some(x) = self.x_minor_ticks {
            options.push(format!("xminorticks={}", if x { "true" } else { "false" }));
        }
        if let Some(x) = self.y_minor_ticks {
            options.push(format!("yminorticks={}", if x { "true" } else { "false" }));
        }
        if let Some(t) = self.ticks {
            options.push(format!("ticks={}", t.render()));
        }

        if self.x_major_grids {
            options.push("xmajorgrids".into());
        }
        if self.y_major_grids {
            options.push("ymajorgrids".into());
        }
        if self.x_minor_grids {
            options.push("xminorgrids".into());
        }
        if self.y_minor_grids {
            options.push("yminorgrids".into());
        }
        if let Some(g) = self.grid {
            options.push(format!("grid={}", g.render()));
        }

        if let Some(p) = self.legend_pos {
            options.push(format!("legend pos={}", p.render()));
        }
        if let Some(a) = self.legend_cell_align {
            options.push(format!("legend cell align={}", a.render()));
        }
        if self.area_legend {
            options.push("area legend".into());
        }

        if self.trim_axis_left {
            options.push("trim axis left".into());
        }
        if self.trim_axis_right {
            options.push("trim axis right".into());
        }
        if let Some(a) = self.axis_x_line_star {
            options.push(format!("axis x line*={}", a.render()));
        }
        if let Some(a) = self.axis_x_line {
            options.push(format!("axis x line={}", a.render()));
        }
        if let Some(a) = self.axis_y_line_star {
            options.push(format!("axis y line*={}", a.render()));
        }
        if let Some(a) = self.axis_y_line {
            options.push(format!("axis y line={}", a.render()));
        }

        if self.forget_plot {
            options.push("forget plot".into());
        }

        // Style

        if let Some(c) = &self.color {
            options.push(format!("color={}", c));
        }
        if let Some(c) = &self.draw {
            options.push(format!("draw={}", c));
        }
        if let Some(c) = &self.fill {
            options.push(format!("fill={}", c));
        }

        if let Some(o) = self.opacity {
            options.push(format!("opacity={}", o));
        }
        if let Some(o) = self.draw_opacity {
            options.push(format!("draw opacity={}", o));
        }
        if let Some(o) = self.fill_opacity {
            options.push(format!("fill opacity={}", o));
        }
        if let Some(o) = self.text_opacity {
            options.push(format!("text opacity={}", o));
        }

        if let Some(l) = &self.line_width {
            options.push(format!("line width={}", l.render()));
        }

        if self.solid {
            options.push("solid".into());
        }
        if let Some(m) = &self.mark {
            options.push(format!("mark={{{}}}", m));
        }
        if let Some(s) = &self.mark_size {
            options.push(format!("mark size={}", s.render()));
        }
        if let Some(s) = &self.mark_options {
            options.push(format!("mark options={{{}}}", s.render().join(",")));
        }

        if let Some(s) = &self.major_tick_length {
            options.push(format!("major tick length={}", s.render()));
        }
        if let Some(s) = &self.minor_tick_length {
            options.push(format!("minor tick length={}", s.render()));
        }

        if let Some(s) = &self.inner_sep {
            options.push(format!("inner sep={}", s.render()));
        }
        if let Some(s) = &self.inner_xsep {
            options.push(format!("inner xsep={}", s.render()));
        }
        if let Some(s) = &self.inner_ysep {
            options.push(format!("inner ysep={}", s.render()));
        }
        if let Some(s) = &self.outer_sep {
            options.push(format!("outer sep={}", s.render()));
        }
        if let Some(s) = &self.outer_xsep {
            options.push(format!("outer xsep={}", s.render()));
        }
        if let Some(s) = &self.outer_ysep {
            options.push(format!("outer ysep={}", s.render()));
        }

        if let Some(f) = &self.number_format {
            options.push(f.render());
        }

        for (key, style) in &self.style_overrides {
            options.push(format!("{}={{{}}}", key, style.render().join(",")));
        }

        options
    }
}

#[derive(Clone, Debug)]
pub enum TickLabels {
    Empty,
    Labels(Vec<String>),
}

impl TickLabels {
    pub fn render(&self) -> String {
        use TickLabels::*;
        match self {
            Empty => "\\empty".into(),
            Labels(l) => format!("{{{}}}", l.join(",")),
        }
    }
}

impl Into<TickLabels> for Vec<String> {
    fn into(self) -> TickLabels {
        TickLabels::Labels(self)
    }
}

#[derive(Clone, Debug, Default)]
pub enum TickPositions {
    #[default]
    Auto,
    Data,
    Empty,
    Coordinates(Vec<f64>),
}

impl TickPositions {
    pub fn render(&self) -> String {
        use TickPositions::*;
        match self {
            Auto => "".into(),
            Data => "data".into(),
            Empty => "\\empty".into(),
            Coordinates(c) => {
                format!("{{{}}}", c.iter()
                    .map(f64::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
                )
            }
        }
    }
}

impl From<Vec<f64>> for TickPositions {
    fn from(c: Vec<f64>) -> Self {
        TickPositions::Coordinates(c)
    }
}

impl From<Vec<usize>> for TickPositions {
    fn from(c: Vec<usize>) -> Self {
        TickPositions::Coordinates(c.into_iter().map(|x| x as f64).collect())
    }
}

#[derive(Clone, Debug)]
pub struct Coordinates(Vec<Coordinate>);

impl Coordinates {
    pub fn render(&self) -> String {
        format!("{{{}}}", self.0.iter()
            .map(Coordinate::render)
            .collect::<Vec<_>>()
            .join(",")
        )
    }
}

impl Into<Coordinates> for Vec<Coordinate> {
    fn into(self) -> Coordinates {
        Coordinates(self)
    }
}

#[derive(Clone, Debug)]
pub enum Coordinate {
    /// Plain Coordinates
    ///
    /// Absolute coordinates in the plotting area.
    Plain(f64, f64),
    /// Axis CS (Axis Coordinate System)
    ///
    /// Coordinates based on the axis limits.
    AxisCs(f64, f64),
    /// Rel Axis CS (Relative Axis Coordinate System)
    ///
    /// Relative coordinates where (0,0) corresponds to the lower left
    /// corner of the axis and (1,1) to the upper right corner.
    RelAxisCs(f64, f64),
    /// LaTeX code for computing coordinates.
    Code(String),
}

impl Coordinate {
    pub fn render(&self) -> String {
        use Coordinate::*;
        match self {
            Plain(x, y) => format!("({},{})", x, y),
            AxisCs(x, y) => format!("(axis cs:{},{})", x, y),
            RelAxisCs(x, y) => format!("(rel axis cs:{},{})", x, y),
            Code(c) => c.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Anchor {
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
    OuterNorthEast,
    OuterNorthWest,
    OuterSouthEast,
    OuterSouthWest,
}

impl Anchor {
    pub fn render(&self) -> String {
        use Anchor::*;
        match self {
            NorthEast => "north east".into(),
            NorthWest => "north west".into(),
            SouthEast => "south east".into(),
            SouthWest => "south west".into(),
            OuterNorthEast => "outer north east".into(),
            OuterNorthWest => "outer north west".into(),
            OuterSouthEast => "outer south east".into(),
            OuterSouthWest => "outer south west".into(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GridLines {
    Major,
    Minor,
    Both,
    None,
}

impl GridLines {
    pub fn render(&self) -> String {
        use GridLines::*;
        match self {
            Major => "major".into(),
            Minor => "minor".into(),
            Both => "both".into(),
            None => "none".into(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TickPos {
    Left,
    Right,
    Both,
}

impl TickPos {
    pub fn render(&self) -> String {
        use TickPos::*;
        match self {
            Left => "left".into(),
            Right => "right".into(),
            Both => "both".into(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TickAlign {
    Inside,
    Center,
    Outside,
}

impl TickAlign {
    pub fn render(&self) -> String {
        use TickAlign::*;
        match self {
            Inside => "inside".into(),
            Center => "center".into(),
            Outside => "outside".into(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CellAlign {
    Left,
    Right,
    Center,
    None,
}

impl CellAlign {
    pub fn render(&self) -> String {
        use CellAlign::*;
        match self {
            Left => "left".into(),
            Right => "right".into(),
            Center => "center".into(),
            None => "none".into(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NumberFormat {
    Fixed(bool),
    Sep(usize, char),
    Precision(usize),
    Relative(usize),
    Sci(bool),
    Frac,
}

impl NumberFormat {
    pub fn render(&self) -> String {
        use NumberFormat::*;
        match self {
            Fixed(zerofill) => {
                if *zerofill {
                    "/pgf/number format/fixed zerofill".into()
                } else {
                    "/pgf/number format/fixed".into()
                }
            }
            Sep(d, c) => format!("/pgf/number format/{} sep={}", d, c),
            Precision(d) => format!("/pgf/number format/precision={}", d),
            Relative(d) => format!("/pgf/number format/relative={}", d),
            Sci(subscript) => {
                if *subscript {
                    "/pgf/number format/sci subscript".into()
                } else {
                    "/pgf/number format/sci".into()
                }
            }
            Frac => "/pgf/number format/frac".into(),
        }
    }
}

/// https://nwalsh.com/tex/texhelp/Plain.html#dimensions
#[derive(Clone, Debug)]
pub enum Dimension {
    /// Point
    Pt(f64),
    /// Pica (1 pc = 12 pt)
    Pc(f64),
    /// Inch (1 in = 72.27 pt)
    In(f64),
    /// Big point (72 bp = 1 inch)
    Bp(f64),
    /// Centimeter
    Cm(f64),
    /// Millimeter
    Mm(f64),
    /// Didor point
    Dd(f64),
    /// Cicero (12 dd)
    Cc(f64),
    /// Scaled point (2^16 sp = 1 pt)
    Sp(f64),
    /// Nominal x-height
    Ex(f64),
    /// Nominal m-width
    Em(f64),
    /// LaTeX code for computing dimension
    Code(String),
}

impl Dimension {
    pub fn render(&self) -> String {
        use Dimension::*;
        match self {
            Pt(v) => format!("{}pt", v),
            Pc(v) => format!("{}pc", v),
            In(v) => format!("{}in", v),
            Bp(v) => format!("{}bp", v),
            Cm(v) => format!("{}cm", v),
            Mm(v) => format!("{}mm", v),
            Dd(v) => format!("{}dd", v),
            Cc(v) => format!("{}cc", v),
            Sp(v) => format!("{}sp", v),
            Em(v) => format!("{}em", v),
            Ex(v) => format!("{}ex", v),
            Code(c) => c.clone(),
        }
    }
}
