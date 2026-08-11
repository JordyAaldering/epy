use std::hash::Hash;

use crate::{data::*, plot::*, tikzir::*};

pub struct TimeSeries<'a, Row, K> {
    series: Vec<TimeSeriesKind<'a, Row, K>>,
    xaxis_label: String,
    yaxis_label: String,
}

enum TimeSeriesKind<'a, Row, K>
{
    Plain {
        df: &'a DataFrame<Row>,
        selector: Box<dyn Fn(&Row) -> f64>,
        label: String,
        color: String,
    },
    Grouped {
        df: &'a DataFrame<Row>,
        group_by: Box<dyn Fn(&Row) -> K>,
        y_selector: Box<dyn Fn(&Row) -> f64>,
    },
}

impl<'a, Row, K> TimeSeries<'a, Row, K>
where
    Row: Clone,
    K: Clone + Eq + Hash + PartialOrd + ToString,
{
    pub fn new(xaxis_label: &str, yaxis_label: &str) -> Self {
        TimeSeries {
            series: Vec::new(),
            xaxis_label: xaxis_label.into(),
            yaxis_label: yaxis_label.into(),
        }
    }

    pub fn series<Y>(mut self, df: &'a DataFrame<Row>, y_selector: Y, label: &str, color: &str) -> Self
    where
        Y: Fn(&Row) -> f64 + 'static,
    {
        self.series.push(TimeSeriesKind::Plain {
            df,
            selector: Box::new(y_selector),
            label: label.into(),
            color: color.into(),
        });
        self
    }

    pub fn grouped_series<G, Y>(
        mut self,
        df: &'a DataFrame<Row>,
        group_by: G,
        y_selector: Y,
    ) -> Self
    where
        G: Fn(&Row) -> K + 'static,
        Y: Fn(&Row) -> f64 + 'static,
    {
        self.series.push(TimeSeriesKind::Grouped {
            df,
            group_by: Box::new(group_by),
            y_selector: Box::new(y_selector),
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
        let mut max_index = 0usize;

        for series in &self.series {
            match &series {
                TimeSeriesKind::Plain {
                    df,
                    selector,
                    label,
                    color,
                } => {
                    let line: Vec<Cs> = df
                        .rows()
                        .iter()
                        .enumerate()
                        .map(|(i, row)| Cs::Plain(i as f64, selector(row)))
                        .collect();

                    if !line.is_empty() {
                        max_index = max_index.max(line.len() - 1);
                        push_series_elements(&mut elements, line, color.clone(), label.clone());
                        emitted_series += 1;
                    }
                }
                TimeSeriesKind::Grouped {
                    df,
                    group_by,
                    y_selector,
                } => {
                    let grouped = (*df).clone().group_by(|row| group_by(row));

                    for gi in 0..grouped.num_groups() {
                        let line: Vec<Cs> = grouped.groups[gi]
                            .iter()
                            .enumerate()
                            .map(|(i, &ri)| {
                                let row = grouped.df.row(ri);
                                Cs::Plain(i as f64, y_selector(row))
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

        Axis { style, data: elements }
    }
}

fn push_series_elements(
    elements: &mut Vec<AxisElement>,
    coordinates: Vec<Cs>,
    color: String,
    label: String,
) {
    let plot_options = StyleBuilder::default()
        .color(color.clone())
        .line_width(Dimension::Pt(1.0))
        .build()
        .unwrap();

    elements.push(AxisElement::AddPlot {
        style: plot_options,
        coordinates,
        closed_cycle: false,
    });

    elements.push(AxisElement::LegendEntry(label));
}
