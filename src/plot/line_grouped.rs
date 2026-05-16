use std::collections::HashMap;

use crate::{data::GroupedFrame, ir::*, plot::{common_axis_options, quartiles}};

/// TODO: actually use the markers, simply increment index for each time series is called
const MARKERS: &[&str] = &["*", "square*", "triangle*", "diamond*", "pentagon*", "o", "square", "triangle"];

pub struct LineGrouped<T> {
    df: GroupedFrame<T>,
    x_selector: Box<dyn Fn(&T) -> f64>,
    y_selector: Box<dyn Fn(&T) -> f64>,
    xaxis_label: String,
    yaxis_label: String,
    ymin: Option<f64>,
    manual_xticks: bool,
    xtick_labels: Option<Vec<String>>,
}

impl<T: Clone> LineGrouped<T> {
    pub fn new(
        df: GroupedFrame<T>,
        x_selector: impl Fn(&T) -> f64 + 'static,
        y_selector: impl Fn(&T) -> f64 + 'static,
        xaxis_label: &str,
        yaxis_label: &str,
    ) -> Self {
        LineGrouped {
            df,
            x_selector: Box::new(x_selector),
            y_selector: Box::new(y_selector),
            xaxis_label: xaxis_label.into(),
            yaxis_label: yaxis_label.into(),
            ymin: Some(0.0),
            manual_xticks: false,
            xtick_labels: None,
        }
    }

    pub fn ymin(mut self, v: Option<f64>) -> Self {
        self.ymin = v;
        self
    }

    pub fn manual_xticks(mut self, enabled: bool) -> Self {
        self.manual_xticks = enabled;
        self
    }

    pub fn xtick_labels(mut self, labels: Vec<String>) -> Self {
        self.manual_xticks = true;
        self.xtick_labels = Some(labels);
        self
    }

    pub fn build_axis(&self) -> Axis {
        // Build the shared x-domain across all groups so every line is aligned to the same ticks.
        let mut unique_x: HashMap<u64, f64> = HashMap::new();
        for gi in 0..self.df.num_groups() {
            for &ri in &self.df.groups[gi] {
                let row = self.df.df.row(ri);
                let x = (self.x_selector)(row);
                unique_x.entry(x.to_bits()).or_insert(x);
            }
        }

        let mut x_keys: Vec<f64> = unique_x.into_values().collect();
        x_keys.sort_by(f64::total_cmp);

        let mut opts = common_axis_options();
        opts.replace(AxisOption::Width("\\epyfigurewidth".into()));
        opts.replace(AxisOption::Height("{\\epyheightratio*\\epyfigurewidth}".into()));
        opts.replace(AxisOption::XLabel(self.xaxis_label.clone()));
        opts.replace(AxisOption::YLabel(self.yaxis_label.clone()));
        if let Some(v) = self.ymin {
            opts.replace(AxisOption::YMin(Numeric::new(v)));
        }

        if self.manual_xticks {
            let ticks: Vec<String> = x_keys.iter().map(ToString::to_string).collect();
            opts.replace(AxisOption::XTicks(ticks));

            let labels: Vec<String> = if let Some(ref lbls) = self.xtick_labels {
                lbls.clone()
            } else {
                x_keys.iter().map(ToString::to_string).collect()
            };
            opts.replace(AxisOption::XTickLabels(labels));
        }

        let mut elements = Vec::new();

        for gi in 0..self.df.num_groups() {
            let mut by_x: HashMap<u64, (f64, Vec<f64>)> = HashMap::new();
            for &ri in &self.df.groups[gi] {
                let row = self.df.df.row(ri);
                let x = (self.x_selector)(row);
                let y = (self.y_selector)(row);
                let entry = by_x.entry(x.to_bits()).or_insert_with(|| (x, Vec::new()));
                entry.1.push(y);
            }

            let mut stats_by_x: Vec<(f64, f64, f64, f64)> = by_x
                .into_values()
                .map(|(x, ys)| {
                    let qs = quartiles(&ys);
                    (x, qs.median, qs.q1, qs.q3)
                })
                .collect();
            stats_by_x.sort_by(|a, b| f64::total_cmp(&a.0, &b.0));

            let cn = format!("epycolorblind{}", gi);
            let marker = MARKERS[gi % MARKERS.len()];

            // Transparent Q1–Q3 band
            let mut band = Vec::new();
            for (x, _, _, q3) in &stats_by_x {
                band.push(Coordinate::Plain(*x, *q3));
            }
            for (x, _, q1, _) in stats_by_x.iter().rev() {
                band.push(Coordinate::Plain(*x, *q1));
            }
            elements.push(AxisElement::Plot(AddPlot {
                opts: vec![format!("fill={}", cn), "fill opacity=0.3".into(), "draw=none".into(), "forget plot".into()],
                coords: band,
                closed_cycle: true,
            }));

            let line: Vec<Coordinate> = stats_by_x
                .iter()
                .map(|(x, median, _, _)| Coordinate::Plain(*x, *median))
                .collect();
            elements.push(AxisElement::Plot(AddPlot {
                opts: vec![
                    cn,
                    format!("mark={marker}"),
                    "mark size=2pt".into(),
                    "mark options={solid,draw=white}".into(),
                    "line width=1pt".into(),
                ],
                coords: line,
                closed_cycle: false,
            }));

            elements.push(AxisElement::LegendEntry(self.df.keys()[gi].to_string()));
        }

        Axis { opts, elements }
    }
}
