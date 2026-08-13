use std::hash::Hash;

use crate::{data::*, plot::*, tikzir::*};

pub struct BarPlot<'a, Row, Category, Group> {
    x_selector: Box<dyn Fn(&Row) -> Category>,
    series: Option<BarSeriesKind<'a, Row, Group>>,
    xaxis_label: String,
    yaxis_label: String,
}

enum BarSeriesKind<'a, Row, Group> {
    Plain {
        df: &'a DataFrame<Row>,
        selector: Box<dyn Fn(&Row) -> f64>,
        aggregation: AggregationMode,
        label: String,
        color: String,
    },
    Grouped {
        df: &'a DataFrame<Row>,
        group_by: Box<dyn Fn(&Row) -> Group>,
        y_selector: Box<dyn Fn(&Row) -> f64>,
        aggregation: AggregationMode,
    },
}

impl<'a, Row, Category, Group> BarPlot<'a, Row, Category, Group>
where
    Row: Clone,
    Category: Clone + Eq + Hash + PartialOrd + ToString,
    Group: Clone + Eq + Hash + PartialOrd + ToString,
{
    pub fn new<X>(x_selector: X, xaxis_label: &str, yaxis_label: &str) -> Self
    where
        X: Fn(&Row) -> Category + 'static,
    {
        BarPlot {
            x_selector: Box::new(x_selector),
            series: None,
            xaxis_label: xaxis_label.into(),
            yaxis_label: yaxis_label.into(),
        }
    }


    pub fn series<Y>(mut self, df: &'a DataFrame<Row>, y_selector: Y, aggregation: AggregationMode, label: &str, color: &str) -> Self
    where
        Y: Fn(&Row) -> f64 + 'static,
    {
        self.series = Some(BarSeriesKind::Plain {
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
        G: Fn(&Row) -> Group + 'static,
        Y: Fn(&Row) -> f64 + 'static,
    {
        self.series = Some(BarSeriesKind::Grouped {
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
        style.ymin = Some(0.0);
        bar_legend_modifier(&mut style);

        let mut data = Vec::new();

        match &self.series {
            Some(BarSeriesKind::Plain { df, selector, aggregation, label, color }) => {
                let grouped = (*df).clone().group_by(|row| (self.x_selector)(row));
                let categories = grouped.keys().to_vec();
                let stats = grouped.summarize_by_group(&**selector, *aggregation);

                let xs: Vec<f64> = (0..categories.len()).map(|i| i as f64).collect();
                push_bar_series(&mut data, &xs, &stats.centers, &stats.lowers, &stats.uppers, 0.7, color, label);

                set_category_axis(&mut style, &categories);
            }
            Some(BarSeriesKind::Grouped { df, group_by, y_selector, aggregation }) => {
                let categories = (*df).clone().group_by(|row| (self.x_selector)(row)).keys().to_vec();

                let grouped_by_hue = (*df).clone().group_by(|row| group_by(row));
                let num_hues = grouped_by_hue.num_groups();
                let bar_width = 0.7 / num_hues as f64;

                for hi in 0..num_hues {
                    let subgroup = grouped_by_hue.subgroup_by(hi, |row| (self.x_selector)(row));
                    let stats = subgroup.summarize_by_group(&**y_selector, *aggregation);
                    let offset = (hi as f64 - (num_hues as f64 - 1.0) / 2.0) * bar_width;

                    // Bars for the same category sit flush against each other since bar_width spans the full slot.
                    let xs: Vec<f64> = stats.keys.iter()
                        .map(|k| {
                            categories.iter().position(|c| c == k).unwrap() as f64 + offset
                        })
                        .collect();

                    let color = format!("colorblind{}", hi);
                    let label = grouped_by_hue.keys()[hi].to_string();
                    push_bar_series(&mut data, &xs, &stats.centers, &stats.lowers, &stats.uppers, bar_width, &color, &label);
                }

                set_category_axis(&mut style, &categories);
            }
            None => {}
        }

        Axis { style, data }
    }
}

fn set_category_axis<C: ToString>(style: &mut Style, categories: &[C]) {
    let n = categories.len();
    style.xticks = (0..n).collect::<Vec<_>>().into();
    style.xtick_labels = Some(categories.iter().map(ToString::to_string).collect::<Vec<_>>().into());
}

fn push_bar_series(
    elements: &mut Vec<AxisElement>,
    xs: &[f64],
    centers: &[f64],
    lowers: &[f64],
    uppers: &[f64],
    bar_width: f64,
    color: &str,
    label: &str,
) {
    let bar_coords: Vec<Cs> = xs.iter().zip(centers.iter())
        .map(|(x, center)| Cs::Plain(*x, *center))
        .collect();

    let plot_options = StyleBuilder::default()
        .ybar(true)
        .bar_width(bar_width)
        .fill(color.to_string())
        .draw("none")
        .area_legend(true)
        .build()
        .unwrap();

    elements.push(AxisElement::AddPlot {
        style: plot_options,
        coordinates: bar_coords,
        closed_cycle: false,
    });
    elements.push(AxisElement::LegendEntry(label.to_string()));

    for ((x, lower), upper) in xs.iter().zip(lowers.iter()).zip(uppers.iter()) {
        let err_options = StyleBuilder::default()
            .color("black!90")
            .line_width(Dimension::Pt(0.9))
            .build()
            .unwrap();
        elements.push(AxisElement::Draw {
            style: err_options,
            from: Cs::Axis(*x, *lower),
            to: Cs::Axis(*x, *upper),
        });
    }
}
