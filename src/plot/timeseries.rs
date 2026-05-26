use crate::{data::*, plot::*, tikzir::*};

pub struct TimeSeries<Row> {
    df: DataFrame<Row>,
    series: Vec<TimeSeriesKind<Row>>,
    xaxis_label: String,
    yaxis_label: String,
}

enum TimeSeriesKind<Row> {
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

impl<Row: Clone> TimeSeries<Row> {
    pub fn new(df: DataFrame<Row>, xaxis_label: &str, yaxis_label: &str) -> Self {
        TimeSeries {
            df,
            series: Vec::new(),
            xaxis_label: xaxis_label.into(),
            yaxis_label: yaxis_label.into(),
        }
    }

    pub fn series<Y>(mut self, y_selector: Y, label: &str, color: &str) -> Self
    where
        Y: Fn(&Row) -> f64 + 'static,
    {
        self.series.push(TimeSeriesKind::Plain {
            selector: Box::new(y_selector),
            label: label.into(),
            color: color.into(),
        });
        self
    }

    pub fn grouped_series<G, Y>(
        mut self,
        group_by: G,
        y_selector: Y,
    ) -> Self
    where
        G: Fn(&Row) -> f64 + 'static,
        Y: Fn(&Row) -> f64 + 'static,
    {
        self.series.push(TimeSeriesKind::Grouped {
            group_by: Box::new(group_by),
            y_selector: Box::new(y_selector),
        });
        self
    }

    pub fn build_axis(&self) -> Axis {
        let mut options = common_axis_options();
        options.xlabel = Some(self.xaxis_label.clone());
        options.ylabel = Some(self.yaxis_label.clone());

        let mut elements = Vec::new();
        let mut emitted_series = 0usize;
        let mut max_index = 0usize;

        for series in &self.series {
            match &series {
                TimeSeriesKind::Plain {
                    selector,
                    label,
                    color,
                } => {
                    let line: Vec<Coordinate> = self.df
                        .rows()
                        .iter()
                        .enumerate()
                        .map(|(i, row)| Coordinate::Plain(i as f64, selector(row)))
                        .collect();

                    if !line.is_empty() {
                        max_index = max_index.max(line.len() - 1);
                        push_series_elements(&mut elements, line, color.clone(), label.clone());
                        emitted_series += 1;
                    }
                }
                TimeSeriesKind::Grouped {
                    group_by,
                    y_selector,
                } => {
                    let grouped = self.df.clone().group_by(|row| group_by(row));

                    for gi in 0..grouped.num_groups() {
                        let line: Vec<Coordinate> = grouped.groups[gi]
                            .iter()
                            .enumerate()
                            .map(|(i, &ri)| {
                                let row = grouped.df.row(ri);
                                Coordinate::Plain(i as f64, y_selector(row))
                            })
                            .collect();

                        if !line.is_empty() {
                            max_index = max_index.max(line.len() - 1);
                            push_series_elements(
                                &mut elements,
                                line,
                                format!("colorblind{}", emitted_series),
                                grouped.keys()[gi].to_string(),
                            );
                            emitted_series += 1;
                        }
                    }
                }
            }
        }

        Axis { style: options, data: elements }
    }
}

fn push_series_elements(
    elements: &mut Vec<AxisElement>,
    coordinates: Vec<Coordinate>,
    color: String,
    label: String,
) {
    let plot_options = StyleBuilder::default()
        .draw(color.clone())
        .line_width(Dimension::Pt(1.0))
        .build()
        .unwrap();

    elements.push(AxisElement::AddPlot {
        options: Some(plot_options),
        coordinates,
        closed_cycle: false,
    });

    elements.push(AxisElement::LegendEntry(label));
}
