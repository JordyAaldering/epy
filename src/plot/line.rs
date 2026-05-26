use crate::{
    data::{DataFrame, GroupedQuartiles},
    ir::*,
    plot::common_axis_options,
};

pub struct LinePlot<Row> {
    df: DataFrame<Row>,
    x_selector: Box<dyn Fn(&Row) -> f64>,
    series: Vec<LineSeriesKind<Row>>,
    xaxis_label: String,
    yaxis_label: String,
}

enum LineSeriesKind<Row> {
    Plain {
        selector: Box<dyn Fn(&Row) -> f64>,
        label: String,
        color: String,
    },
    Grouped {
        group_by: Box<dyn Fn(&Row) -> f64>,
        y_selector: Box<dyn Fn(&Row) -> f64>,
    },
}

impl<Row: Clone> LinePlot<Row> {
    pub fn new<X>(df: DataFrame<Row>, x_selector: X, xaxis_label: &str, yaxis_label: &str) -> Self
    where
        X: Fn(&Row) -> f64 + 'static,
    {
        LinePlot {
            df,
            x_selector: Box::new(x_selector),
            series: Vec::new(),
            xaxis_label: xaxis_label.into(),
            yaxis_label: yaxis_label.into(),
        }
    }

    pub fn series<Y>(mut self, y_selector: Y, label: &str, color: &str) -> Self
    where
        Y: Fn(&Row) -> f64 + 'static,
    {
        self.series.push(LineSeriesKind::Plain {
            selector: Box::new(y_selector),
            label: label.into(),
            color: color.into(),
        });
        self
    }

    pub fn grouped_series<G, Y>(mut self, group_by: G, y_selector: Y) -> Self
    where
        G: Fn(&Row) -> f64 + 'static,
        Y: Fn(&Row) -> f64 + 'static,
    {
        self.series.push(LineSeriesKind::Grouped {
            group_by: Box::new(group_by),
            y_selector: Box::new(y_selector),
        });
        self
    }

    pub fn build_axis(&self) -> Axis {
        let x_grouped = self.df.clone().group_by(|row| (self.x_selector)(row));

        let mut opts = common_axis_options();
        opts.replace(AxisOption::Width("\\epyfigurewidth".into()));
        opts.replace(AxisOption::Height("{\\epyheightratio*\\epyfigurewidth}".into()));
        opts.replace(AxisOption::XLabel(self.xaxis_label.clone()));
        opts.replace(AxisOption::YLabel(self.yaxis_label.clone()));

        let mut elements = Vec::new();
        let mut emitted_series = 0usize;

        for series in &self.series {
            match &series {
                LineSeriesKind::Plain { selector, label, color } => {
                    let stats = x_grouped.quartiles_by_group(&**selector);

                    push_series_elements(
                        &mut elements,
                        &stats,
                        color.clone(),
                        MARKERS[emitted_series % MARKERS.len()].to_string(),
                        label.clone(),
                    );
                    emitted_series += 1;
                }
                LineSeriesKind::Grouped { group_by, y_selector } => {
                    let grouped = self.df.clone().group_by(|row| group_by(row));

                    for gi in 0..grouped.num_groups() {
                        let subgroup = grouped.subgroup_by(gi, |row| (self.x_selector)(row));
                        let stats = subgroup.quartiles_by_group(&**y_selector);

                        push_series_elements(
                            &mut elements,
                            &stats,
                            format!("colorblind{}", emitted_series),
                            MARKERS[emitted_series % MARKERS.len()].to_string(),
                            grouped.keys()[gi].to_string(),
                        );
                        emitted_series += 1;
                    }
                }
            }
        }

        Axis { opts, elements }
    }
}

fn push_series_elements(
    elements: &mut Vec<AxisElement>,
    stats: &GroupedQuartiles,
    color: String,
    marker: String,
    label: String,
) {
    let mut band = Vec::new();
    for (x, q3) in stats.keys.iter().zip(stats.q3s.iter()) {
        band.push(Coordinate::Plain(*x, *q3));
    }
    for (x, q1) in stats.keys.iter().zip(stats.q1s.iter()).rev() {
        band.push(Coordinate::Plain(*x, *q1));
    }
    elements.push(AxisElement::Plot(AddPlot {
        opts: vec![
            format!("fill={}", color),
            "fill opacity=0.3".into(),
            "draw=none".into(),
            "forget plot".into(),
        ],
        coords: band,
        closed_cycle: true,
    }));

    let line: Vec<Coordinate> = stats.keys
        .iter()
        .zip(stats.medians.iter())
        .map(|(x, median)| Coordinate::Plain(*x, *median))
        .collect();
    elements.push(AxisElement::Plot(AddPlot {
        opts: vec![
            color,
            "line width=1pt".into(),
            format!("mark={}", marker),
            format!("mark size={}pt", MARK_SIZE_PT),
            format!("mark options={{solid,draw=white,line width=-{}pt}}", MARK_OUTLINE_PT),
        ],
        coords: line,
        closed_cycle: false,
    }));

    elements.push(AxisElement::LegendEntry(label));
}
