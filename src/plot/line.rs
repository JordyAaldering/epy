use crate::{data::DataFrame, ir::*, plot::{common_axis_options, quartiles}};

pub struct LinePlot<T> {
    df: DataFrame<T>,
    x_selector: Box<dyn Fn(&T) -> f64>,
    series: Vec<LineSeries<T>>,
    xaxis_label: String,
    yaxis_label: String,
    ymin: Option<f64>,
    manual_xticks: bool,
    xtick_labels: Option<Vec<String>>,
}

struct LineSeries<T> {
    selector: Box<dyn Fn(&T) -> f64>,
    label: String,
    color: String,
    marker: String,
}

impl<T: Clone> LinePlot<T> {
    pub fn new(
        df: DataFrame<T>,
        x_selector: impl Fn(&T) -> f64 + 'static,
        xaxis_label: &str,
        yaxis_label: &str,
    ) -> Self {
        LinePlot {
            df,
            x_selector: Box::new(x_selector),
            series: Vec::new(),
            xaxis_label: xaxis_label.into(),
            yaxis_label: yaxis_label.into(),
            ymin: Some(0.0),
            manual_xticks: false,
            xtick_labels: None,
        }
    }

    pub fn series(
        mut self,
        y_selector: impl Fn(&T) -> f64 + 'static,
        label: &str,
        color: &str,
    ) -> Self {
        let marker = MARKERS[self.series.len() % MARKERS.len()].to_string();
        self.series.push(LineSeries {
            selector: Box::new(y_selector),
            label: label.into(),
            color: color.into(),
            marker,
        });
        self
    }

    pub fn grouped_series<F>(
        mut self,
        group_by: F,
        y_selector: F,
        // add fields if needed
    ) -> Self
    where
        F: Fn(&T) -> f64,
    {
        todo!()
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
        let grouped = self.df.clone().group_by(|row| (self.x_selector)(row));
        let keys = grouped.keys().to_vec();
        let all_stats: Vec<(Vec<f64>, Vec<f64>, Vec<f64>)> = self
            .series
            .iter()
            .map(|s| {
                let mut meds = Vec::with_capacity(grouped.num_groups());
                let mut q1s = Vec::with_capacity(grouped.num_groups());
                let mut q3s = Vec::with_capacity(grouped.num_groups());

                for gi in 0..grouped.num_groups() {
                    let vals = grouped.group_values(gi, &*s.selector);
                    let qs = quartiles(&vals);
                    meds.push(qs.median);
                    q1s.push(qs.q1);
                    q3s.push(qs.q3);
                }

                (meds, q1s, q3s)
            })
            .collect();

        let mut opts = common_axis_options();
        opts.replace(AxisOption::Width("\\epyfigurewidth".into()));
        opts.replace(AxisOption::Height("{\\epyheightratio*\\epyfigurewidth}".into()));
        opts.replace(AxisOption::XLabel(self.xaxis_label.clone()));
        opts.replace(AxisOption::YLabel(self.yaxis_label.clone()));
        if let Some(v) = self.ymin {
            opts.replace(AxisOption::YMin(Numeric::new(v)));
        }

        if self.manual_xticks {
            let ticks: Vec<String> = keys.iter().map(ToString::to_string).collect();
            opts.replace(AxisOption::XTicks(ticks));

            let labels: Vec<String> = if let Some(ref lbls) = self.xtick_labels {
                lbls.clone()
            } else {
                keys.iter().map(ToString::to_string).collect()
            };
            opts.replace(AxisOption::XTickLabels(labels));
        }

        let mut elements = Vec::new();

        for (si, series) in self.series.iter().enumerate() {
            let (meds, q1s, q3s) = &all_stats[si];
            let cn = series.color.to_string();

            // Transparent Q1–Q3 band
            let mut band = Vec::new();
            for (x, q3) in keys.iter().zip(q3s.iter()) {
                band.push(Coordinate::Plain(*x, *q3));
            }
            for (x, q1) in keys.iter().zip(q1s.iter()).rev() {
                band.push(Coordinate::Plain(*x, *q1));
            }
            elements.push(AxisElement::Plot(AddPlot {
                opts: vec![format!("fill={}", cn), "fill opacity=0.3".into(), "draw=none".into(), "forget plot".into()],
                coords: band,
                closed_cycle: true,
            }));

            // Median line
            let line: Vec<Coordinate> = keys.iter().zip(meds.iter())
                .map(|(x, median)| Coordinate::Plain(*x, *median))
                .collect();
            elements.push(AxisElement::Plot(AddPlot {
                opts: vec![
                    cn,
                    "line width=1pt".into(),
                    format!("mark={}", series.marker),
                    format!("mark size={}pt", MARK_SIZE_PT),
                    format!("mark options={{solid,draw=white,line width=-{}pt}}", MARK_OUTLINE_PT),
                ],
                coords: line,
                closed_cycle: false,
            }));

            elements.push(AxisElement::LegendEntry(series.label.clone()));
        }

        Axis { opts, elements }
    }
}
