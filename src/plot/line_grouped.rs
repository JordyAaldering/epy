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
            xtick_labels: None,
        }
    }

    pub fn ymin(mut self, v: Option<f64>) -> Self {
        self.ymin = v;
        self
    }

    pub fn xtick_labels(mut self, labels: Vec<String>) -> Self {
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

        let x_to_index: HashMap<u64, usize> = x_keys
            .iter()
            .enumerate()
            .map(|(i, &x)| (x.to_bits(), i))
            .collect();

        let n = x_keys.len();

        let mut opts = common_axis_options();
        opts.replace(AxisOption::Width("\\epyfigurewidth".into()));
        opts.replace(AxisOption::Height("{\\dimexpr\\epyheightratio\\epyfigurewidth\\relax}".into()));
        opts.replace(AxisOption::XLabel(self.xaxis_label.clone()));
        opts.replace(AxisOption::YLabel(self.yaxis_label.clone()));
        if let Some(v) = self.ymin {
            opts.replace(AxisOption::YMin(Numeric::new(v)));
        }

        let tick_str: Vec<String> = (0..n).map(|i| i.to_string()).collect();
        opts.replace(AxisOption::XTicks(tick_str));

        let labels: Vec<String> = if let Some(ref lbls) = self.xtick_labels {
            lbls.clone()
        } else {
            x_keys.iter().map(ToString::to_string).collect()
        };
        opts.replace(AxisOption::XTickLabels(labels));
        opts.replace(AxisOption::XMin(Numeric::new(-0.5)));
        opts.replace(AxisOption::XMax(Numeric::new(n as f64 - 0.5)));

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

            let mut stats_by_index: Vec<(usize, f64, f64, f64)> = by_x
                .into_values()
                .map(|(x, ys)| {
                    let qs = quartiles(&ys);
                    (x_to_index[&x.to_bits()], qs.median, qs.q1, qs.q3)
                })
                .collect();
            stats_by_index.sort_by_key(|(xi, _, _, _)| *xi);

            let cn = format!("epycolorblind{}", gi);
            let marker = MARKERS[gi % MARKERS.len()];

            // Transparent Q1–Q3 band
            let mut band = Vec::new();
            for (xi, _, _, q3) in &stats_by_index {
                band.push(Coordinate::Plain(*xi as f64, *q3));
            }
            for (xi, _, q1, _) in stats_by_index.iter().rev() {
                band.push(Coordinate::Plain(*xi as f64, *q1));
            }
            elements.push(AxisElement::Plot(AddPlot {
                opts: vec![format!("fill={}", cn), "fill opacity=0.3".into(), "draw=none".into(), "forget plot".into()],
                coords: band,
                closed_cycle: true,
            }));

            let line: Vec<Coordinate> = stats_by_index
                .iter()
                .map(|(xi, median, _, _)| Coordinate::Plain(*xi as f64, *median))
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
