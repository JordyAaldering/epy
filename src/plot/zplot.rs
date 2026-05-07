use crate::{color::Color, data::GroupedFrame, ir::*, plot::common_axis_options};

const MARKERS: &[&str] = &["*", "square*", "triangle*", "diamond*", "pentagon*", "o", "square", "triangle"];

pub struct ZPlot {
    df: GroupedFrame,
    x_col: String,
    y_col: String,
    hue_col: String,
    xaxis_label: String,
    yaxis_label: String,
}

impl ZPlot {
    pub fn new(
        df: GroupedFrame,
        x_col: &str,
        y_col: &str,
        hue_col: &str,
        xaxis_label: &str,
        yaxis_label: &str,
    ) -> Self {
        ZPlot {
            df,
            x_col: x_col.into(),
            y_col: y_col.into(),
            hue_col: hue_col.into(),
            xaxis_label: xaxis_label.into(),
            yaxis_label: yaxis_label.into(),
        }
    }

    pub fn build_document(&self) -> PlotDocument {
        let ax = self.build_axis();
        PlotDocument::new(Vec::new(), ax, None)
    }

    fn build_axis(&self) -> Axis {
        let n = self.df.num_groups();
        let color_names: Vec<String> = (0..n).map(|i| Color::Colorblind(i).tikz_name()).collect();

        let mut opts = common_axis_options();
        opts.push(AxisOption::key_value("x grid style", format!("{{{}}}", Color::Grid.tikz_name())));
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

            let coordinates = self.group_coordinates(gi);
            elements.push(AxisElement::Plot(AddPlot {
                opts: vec![
                    cn.clone(),
                    format!("mark={marker}"),
                    "mark size=2pt".into(),
                    "mark options={solid,draw=white}".into(),
                    "line width=1pt".into(),
                ],
                coords: coordinates,
                closed_cycle: false,
            }));
            elements.push(AxisElement::LegendEntry(label));
        }

        Axis { opts, elements }
    }

    fn group_coordinates(&self, gi: usize) -> Vec<Coordinate> {
        let hue_values = self.df.group_values(gi, &self.hue_col);
        let xs = self.df.group_values(gi, &self.x_col);
        let ys = self.df.group_values(gi, &self.y_col);

        let mut rows: Vec<(f64, f64, f64)> = hue_values
            .into_iter()
            .zip(xs)
            .zip(ys)
            .map(|((hue, x), y)| (hue, x, y))
            .collect();
        rows.sort_by(|left, right| left.0.total_cmp(&right.0));

        let mut coordinates = Vec::new();
        let mut idx = 0;
        while idx < rows.len() {
            let hue_bits = rows[idx].0.to_bits();
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let mut count = 0usize;

            while idx < rows.len() && rows[idx].0.to_bits() == hue_bits {
                sum_x += rows[idx].1;
                sum_y += rows[idx].2;
                count += 1;
                idx += 1;
            }

            coordinates.push(Coordinate::Plain(sum_x / count as f64, sum_y / count as f64));
        }

        coordinates
    }
}
