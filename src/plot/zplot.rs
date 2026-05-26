use std::collections::HashMap;

use crate::{data::DataFrame, ir::*, plot::{common_axis_options, median}};

pub struct ZPlot<Row> {
    df: DataFrame<Row>,
    series_selector: Box<dyn Fn(&Row) -> f64>,
    hue_selector: Box<dyn Fn(&Row) -> f64>,
    x_selector: Box<dyn Fn(&Row) -> f64>,
    y_selector: Box<dyn Fn(&Row) -> f64>,
    /// Filter elements after median aggregation
    agg_filter: Option<Box<dyn Fn(f64, f64, f64) -> bool>>,
    xaxis_label: String,
    yaxis_label: String,
}

impl<Row: Clone> ZPlot<Row> {
    /// Create a scatter plot where each unique value of `series_selector` becomes a
    /// separate series in the legend. Within each series, rows are grouped by
    /// `hue_selector` and the mean `(x_selector, y_selector)` is plotted per group.
    pub fn new<G, H, X, Y>(
        df: DataFrame<Row>,
        series_selector: G,
        hue_selector: H,
        x_selector: X,
        y_selector: Y,
        xaxis_label: &str,
        yaxis_label: &str,
    ) -> Self
    where
        G: Fn(&Row) -> f64 + 'static,
        H: Fn(&Row) -> f64 + 'static,
        X: Fn(&Row) -> f64 + 'static,
        Y: Fn(&Row) -> f64 + 'static,
    {
        ZPlot {
            df,
            series_selector: Box::new(series_selector),
            hue_selector: Box::new(hue_selector),
            x_selector: Box::new(x_selector),
            y_selector: Box::new(y_selector),
            agg_filter: None,
            xaxis_label: xaxis_label.into(),
            yaxis_label: yaxis_label.into(),
        }
    }

    /// Filter elements after median aggregation
    pub fn with_filter(mut self, filter: impl Fn(f64, f64, f64) -> bool + 'static) -> Self {
        self.agg_filter = Some(Box::new(filter));
        self
    }

    pub fn build_axis(&self) -> Axis {
        let grouped = self.df.clone().group_by(|row| (self.series_selector)(row));
        let mut series_groups: Vec<(f64, Vec<Coordinate>)> = Vec::new();

        for gi in 0..grouped.num_groups() {
            let series_key = grouped.keys()[gi];
            let mut by_hue: HashMap<u64, (f64, Vec<f64>, Vec<f64>)> = HashMap::new();

            for &ri in &grouped.groups[gi] {
                let row = grouped.df.row(ri);
                let hue = (self.hue_selector)(row);
                let x = (self.x_selector)(row);
                let y = (self.y_selector)(row);
                let entry = by_hue
                    .entry(hue.to_bits())
                    .or_insert_with(|| (hue, Vec::new(), Vec::new()));
                entry.1.push(x);
                entry.2.push(y);
            }

            let mut medians_by_hue: Vec<(f64, f64, f64)> = by_hue
                .into_values()
                .map(|(hue, xs, ys)| (hue, median(&xs), median(&ys)))
                .filter(|(hue, med_x, med_y)| {
                    if let Some(filter) = &self.agg_filter {
                        filter(*hue, *med_x, *med_y)
                    } else {
                        true
                    }
                })
                .collect();
            medians_by_hue.sort_by(|a, b| f64::total_cmp(&a.0, &b.0));

            let coords = medians_by_hue
                .into_iter()
                .map(|(_, x, y)| Coordinate::Plain(x, y))
                .collect();
            series_groups.push((series_key, coords));
        }

        let mut opts = common_axis_options();
        opts.replace(AxisOption::SetXGridColor);
        opts.replace(AxisOption::XMajorGrids(true));
        opts.replace(AxisOption::Width("\\epyfigurewidth".into()));
        opts.replace(AxisOption::Height("{\\epyheightratio*\\epyfigurewidth}".into()));
        opts.replace(AxisOption::XLabel(self.xaxis_label.clone()));
        opts.replace(AxisOption::YLabel(self.yaxis_label.clone()));
        opts.replace(AxisOption::YMin(Numeric::new(0.0)));

        let mut elements = Vec::new();
        for (gi, (key, coords)) in series_groups.into_iter().enumerate() {
            let cn = format!("colorblind{}", gi);
            let marker = MARKERS[gi % MARKERS.len()];
            elements.push(AxisElement::Plot(AddPlot {
                opts: vec![
                    cn.into(),
                    "line width=1pt".into(),
                    format!("mark={}", marker),
                    format!("mark size={}pt", MARK_SIZE_PT),
                    format!("mark options={{solid,draw=white,line width=-{}pt}}", MARK_OUTLINE_PT),
                ],
                coords,
                closed_cycle: false,
            }));
            elements.push(AxisElement::LegendEntry(key.to_string()));
        }

        Axis { opts, elements }
    }
}
