use crate::{color::Color, data::GroupedFrame, ir::*, plot::common_axis_options};

const MARKERS: &[&str] = &["*", "square*", "triangle*", "diamond*", "pentagon*", "o", "square", "triangle"];

pub struct ZPlot {
    df: GroupedFrame,
    x_col: String,
    y_col: String,
    xaxis_label: String,
    yaxis_label: String,
}

impl ZPlot {
    pub fn new(
        df: GroupedFrame,
        x_col: &str,
        y_col: &str,
        xaxis_label: &str,
        yaxis_label: &str,
    ) -> Self {
        ZPlot {
            df,
            x_col: x_col.into(),
            y_col: y_col.into(),
            xaxis_label: xaxis_label.into(),
            yaxis_label: yaxis_label.into(),
        }
    }

    pub fn render(&self) -> String {
        self.build_document().render_tikz()
    }

    fn build_document(&self) -> PlotDocument {
        let ax = self.build_axis();
        PlotDocument::new(Vec::new(), ax, None)
    }

    fn build_axis(&self) -> Axis {
        let n = self.df.num_groups();
        let color_names: Vec<String> = (0..n).map(|i| Color::Colorblind(i).tikz_name()).collect();

        let mut opts = common_axis_options();
        opts.push(AxisOption::key_value("x grid style", "{epygridcolor}"));
        opts.push(AxisOption::flag("xmajorgrids"));
        opts.push(AxisOption::key_value("width", "\\epyfigurewidth"));
        opts.push(AxisOption::key_value("height", "\\epyfigureheight"));
        opts.push(AxisOption::key_value("xlabel", format!("{{\\epylabelsize {}}}", self.xaxis_label)));
        opts.push(AxisOption::key_value("ylabel", format!("{{\\epylabelsize {}}}", self.yaxis_label)));
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
            let xs = self.df.group_values(gi, &self.x_col);
            let ys = self.df.group_values(gi, &self.y_col);

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
