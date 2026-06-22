use crate::{data::*, plot::*, tikzir::*};

pub struct LinePlot<'a, Row> {
    x_selector: Box<dyn Fn(&Row) -> f64>,
    series: Vec<LineSeriesKind<'a, Row>>,
    xaxis_label: String,
    yaxis_label: String,
}

enum LineSeriesKind<'a, Row> {
    Plain {
        df: &'a DataFrame<Row>,
        selector: Box<dyn Fn(&Row) -> f64>,
        aggregation: AggregationMode,
        label: String,
        color: String,
    },
    Grouped {
        df: &'a DataFrame<Row>,
        group_by: Box<dyn Fn(&Row) -> f64>,
        y_selector: Box<dyn Fn(&Row) -> f64>,
        aggregation: AggregationMode,
    },
}

impl<'a, Row: Clone> LinePlot<'a, Row> {
    pub fn new<X>(x_selector: X, xaxis_label: &str, yaxis_label: &str) -> Self
    where
        X: Fn(&Row) -> f64 + 'static,
    {
        LinePlot {
            x_selector: Box::new(x_selector),
            series: Vec::new(),
            xaxis_label: xaxis_label.into(),
            yaxis_label: yaxis_label.into(),
        }
    }

    pub fn series<Y>(mut self, df: &'a DataFrame<Row>, y_selector: Y, aggregation: AggregationMode, label: &str, color: &str) -> Self
    where
        Y: Fn(&Row) -> f64 + 'static,
    {
        self.series.push(LineSeriesKind::Plain {
            df,
            selector: Box::new(y_selector),
            aggregation,
            label: label.into(),
            color: color.into(),
        });
        self
    }

    pub fn grouped_series<G, Y>(mut self, df: &'a DataFrame<Row>, group_by: G, y_selector: Y, aggregation: AggregationMode) -> Self
    where
        G: Fn(&Row) -> f64 + 'static,
        Y: Fn(&Row) -> f64 + 'static,
    {
        self.series.push(LineSeriesKind::Grouped {
            df,
            group_by: Box::new(group_by),
            y_selector: Box::new(y_selector),
            aggregation,
        });
        self
    }

    pub fn build_axis(&self) -> Axis {
        let mut style = common_axis_style();
        style.xlabel = Some(self.xaxis_label.clone());
        style.ylabel = Some(self.yaxis_label.clone());
        line_legend_modifier(&mut style);

        let mut elements = Vec::new();
        let mut emitted_series = 0usize;

        for series in &self.series {
            match &series {
                LineSeriesKind::Plain { df, selector, aggregation, label, color } => {
                    let x_grouped = (*df).clone().group_by(|row| (self.x_selector)(row));
                    let stats = x_grouped.summarize_by_group(&**selector, *aggregation);

                    push_series_elements(
                        &mut elements,
                        &stats,
                        color.clone(),
                        MARKERS[emitted_series % MARKERS.len()].to_string(),
                        label.clone(),
                    );
                    emitted_series += 1;
                }
                LineSeriesKind::Grouped { df, group_by, y_selector, aggregation } => {
                    let grouped = (*df).clone().group_by(|row| group_by(row));

                    for gi in 0..grouped.num_groups() {
                        let subgroup = grouped.subgroup_by(gi, |row| (self.x_selector)(row));
                        let stats = subgroup.summarize_by_group(&**y_selector, *aggregation);

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

        Axis { style, data: elements }
    }
}

fn push_series_elements(
    elements: &mut Vec<AxisElement>,
    stats: &GroupedSummaryBand,
    color: String,
    marker: String,
    label: String,
) {
    let mut band = Vec::new();
    for (x, upper) in stats.keys.iter().zip(stats.uppers.iter()) {
        band.push(Cs::Plain(*x, *upper));
    }
    for (x, lower) in stats.keys.iter().zip(stats.lowers.iter()).rev() {
        band.push(Cs::Plain(*x, *lower));
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

    let line: Vec<Cs> = stats.keys
        .iter()
        .zip(stats.centers.iter())
        .map(|(x, center)| Cs::Plain(*x, *center))
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
