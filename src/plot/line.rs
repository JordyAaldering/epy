use crate::{data::*, plot::*, tikzir::*};

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

        let mut options = common_axis_options();
        options.xlabel = Some(self.xaxis_label.clone());
        options.ylabel = Some(self.yaxis_label.clone());

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

        Axis { style: options, data: elements }
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

    let err_options = StyleBuilder::default()
        .draw("none")
        .fill(color.clone())
        .fill_opacity(0.3)
        .forget_plot(true)
        .build()
        .unwrap();

    elements.push(AxisElement::AddPlot {
        style: err_options,
        coordinates: band,
        closed_cycle: true,
    });

    let line: Vec<Coordinate> = stats.keys
        .iter()
        .zip(stats.medians.iter())
        .map(|(x, median)| Coordinate::Plain(*x, *median))
        .collect();

    let plot_options = StyleBuilder::default()
        .color(color.clone())
        .line_width(Dimension::Pt(1.0))
        .mark(marker.clone())
        .mark_size(Dimension::Pt(MARK_SIZE_PT))
        .mark_options(StyleBuilder::default()
            .solid(true)
            .draw("white".to_string())
            .line_width(Dimension::Pt(-MARK_OUTLINE_PT))
            .build()
            .unwrap()
        )
        .build()
        .unwrap();

    elements.push(AxisElement::AddPlot {
        style: plot_options,
        coordinates: line,
        closed_cycle: false,
    });

    elements.push(AxisElement::LegendEntry(label));
}
