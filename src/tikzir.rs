#[derive(Clone, Debug)]
pub struct Axis {
    pub options: AxisOptions,
    pub style: StyleOptions,
    pub extra_styles: ExtraStyleOptions,
    pub elements: Vec<AxisElement>,
}

impl Axis {
    pub fn render(&self) -> String {
        let mut axis_options = self.options.render();
        axis_options.append(&mut self.style.render());
        axis_options.append(&mut self.extra_styles.render());

        let mut res = String::new();
        res.push_str("\\begin{axis}\n");
        if !axis_options.is_empty() {
            res.push('[');
            res.push_str(&axis_options.into_iter()
                .map(|s| format!("  {},\n", s))
                .collect::<String>());
            res.push(']');
        }
        res.push_str("\\end{axis}");
        res
    }
}

#[derive(Clone, Debug)]
pub enum AxisElement {
    Plot,
    LegendEntry,
    LegendImage,
}

#[derive(Clone, Debug)]
pub struct AxisOptions {
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,

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
    pub xticks: TickCoordinates,
    pub yticks: TickCoordinates,

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
    pub xtick_labels: Option<Vec<String>>,
    pub ytick_labels: Option<Vec<String>>,
    /// As xticklabels provides explicit tick labels for each xtick,
    /// the key extra x tick labels provides explicit tick labels
    /// for every element in extra x ticks.
    pub extra_xtick_labels: Option<Vec<String>>,
    pub extra_ytick_labels: Option<Vec<String>>,

    /// Allows to choose where to place the small tick lines. In the default
    /// configuration, this does also affect tick labels.
    pub xtick_pos: Option<TickPos>,
    pub ytick_pos: Option<TickPos>,

    /// Allows to change the location of the ticks relative to the axis lines.
    pub xtick_align: Option<TickAlign>,
    pub ytick_align: Option<TickAlign>,

    /// Enables/disables different grid lines. Major grid lines are placed at
    /// the normal tick positions (see [`x_major_ticks`]) while minor grid
    /// lines are placed at minor ticks (see [`x_minor_ticks`]`).
    pub x_major_grids: bool,
    pub y_major_grids: bool,
    pub x_minor_grids: bool,
    pub y_minor_grids: bool,

    pub legend_pos: Option<LegendPos>,
}

impl AxisOptions {
    pub fn render(&self) -> Vec<String> {
        let mut options = Vec::new();

        if let Some(width) = self.width {
            options.push(format!("width={}", width.render()));
        }
        if let Some(height) = self.height {
            options.push(format!("height={}", height.render()));
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

        if !matches!(self.xticks, TickCoordinates::Auto) {
            options.push(format!("xtick={}", self.xticks.render()));
        }
        if !matches!(self.yticks, TickCoordinates::Auto) {
            options.push(format!("ytick={}", self.yticks.render()));
        }

        if let Some(c) = &self.extra_xticks {
            options.push(format!("extra x ticks={}", c.render()));
        }
        if let Some(c) = &self.extra_yticks {
            options.push(format!("extra y ticks={}", c.render()));
        }

        if let Some(l) = &self.xtick_labels {
            options.push(format!("x tick labels={{{}}}", l.join(",")));
        }
        if let Some(l) = &self.ytick_labels {
            options.push(format!("y tick labels={{{}}}", l.join(",")));
        }
        if let Some(l) = &self.extra_xtick_labels {
            options.push(format!("extra x tick labels={{{}}}", l.join(",")));
        }
        if let Some(l) = &self.extra_ytick_labels {
            options.push(format!("extra y tick labels={{{}}}", l.join(",")));
        }

        if let Some(p) = &self.xtick_pos {
            options.push(format!("x tick pos={}", p.render()));
        }
        if let Some(p) = &self.ytick_pos {
            options.push(format!("y tick pos={}", p.render()));
        }

        if let Some(a) = &self.xtick_align {
            options.push(format!("x tick align={}", a.render()));
        }
        if let Some(a) = &self.ytick_align {
            options.push(format!("y tick align={}", a.render()));
        }

        if self.x_major_grids {
            options.push("x major grids".into());
        }
        if self.y_major_grids {
            options.push("y major grids".into());
        }
        if self.x_minor_grids {
            options.push("x minor grids".into());
        }
        if self.y_minor_grids {
            options.push("y minor grids".into());
        }

        if let Some(p) = self.legend_pos {
            options.push(format!("legend pos={}", p.render()));

        }

        options
    }
}

#[derive(Clone, Debug)]
pub struct StyleOptions {
    pub opacity: Option<f64>,
    pub text_opacity: Option<f64>,
}

impl StyleOptions {
    pub fn render(&self) -> Vec<String> {
        let mut options = Vec::new();

        if let Some(o) = self.opacity {
            options.push(format!("opacity={}", o));
        }
        if let Some(o) = self.text_opacity {
            options.push(format!("text opacity={}", o));
        }

        options
    }
}

#[derive(Clone, Debug)]
pub struct ExtraStyleOptions {
    /// An abbreviation for every extra x tick/.append style={〈key-value-list〉}.
    pub extra_xtick_style: Option<StyleOptions>,
    pub extra_ytick_style: Option<StyleOptions>,
}

impl ExtraStyleOptions {
    pub fn render(&self) -> Vec<String> {
        let mut options = Vec::new();

        if let Some(s) = &self.extra_xtick_style {
            options.push(format!("extra x tick style={{{}}}", s.render().join(",")));
        }
        if let Some(s) = &self.extra_ytick_style {
            options.push(format!("extra y tick style={{{}}}", s.render().join(",")));
        }

        options
    }
}

#[derive(Clone, Debug, Default)]
pub enum TickCoordinates {
    #[default]
    Auto,
    Data,
    Empty,
    Coordinates(Coordinates),
}

impl TickCoordinates {
    pub fn render(&self) -> String {
        use TickCoordinates::*;
        match self {
            Auto => "".into(),
            Data => "data".into(),
            Empty => "\\empty".into(),
            Coordinates(c) => c.render(),
        }
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

#[derive(Clone, Copy, Debug)]
pub enum Coordinate {
    Plain(f64, f64),
    /// Draw at axis coordinates.
    AxisCs(f64, f64),
}

impl Coordinate {
    pub fn render(&self) -> String {
        use Coordinate::*;
        match self {
            Plain(x, y) => format!("({},{})", x, y),
            AxisCs(x, y) => format!("axis cs:({},{})", x, y),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LegendPos {
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
    OuterNorthEast,
    OuterNorthWest,
    OuterSouthEast,
    OuterSouthWest,
}

impl LegendPos {
    pub fn render(&self) -> String {
        use LegendPos::*;
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

/// https://nwalsh.com/tex/texhelp/Plain.html#dimensions
#[derive(Clone, Copy, Debug)]
pub enum Dimension {
    /// Nominal m-width.
    Em(f64),
    /// Nominal x-height.
    Ex(f64),
    /// Point.
    Pt(f64),
    /// Millimeter.
    Mm(f64),
}

impl Dimension {
    pub fn render(&self) -> String {
        use Dimension::*;
        match self {
            Em(v) => format!("{}em", v),
            Ex(v) => format!("{}ex", v),
            Pt(v) => format!("{}pt", v),
            Mm(v) => format!("{}mm", v),
        }
    }
}
