use crate::{data::DataFrame, ir::*, plot::{common_axis_options, quartiles}};

struct LineSeries<T> {
    selector: Box<dyn Fn(&T) -> f64>,
    label: String,
    color: String,
}

pub struct LinePlot<T> {
    df: DataFrame<T>,
    x_selector: Box<dyn Fn(&T) -> f64>,
    series: Vec<LineSeries<T>>,
    xaxis_label: String,
    yaxis_label: String,
    ymin: Option<f64>,
    xtick_labels: Option<Vec<String>>,
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
            xtick_labels: None,
        }
    }

    pub fn series(
        mut self,
        selector: impl Fn(&T) -> f64 + 'static,
        label: &str,
        color: String,
) -> Self {
        self.series.push(LineSeries {
            selector: Box::new(selector),
            label: label.into(),
            color,
        });
        self
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

        let n = keys.len();

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
            keys.iter().map(ToString::to_string).collect()
        };
        opts.replace(AxisOption::XTickLabels(labels));
        opts.replace(AxisOption::XMin(Numeric::new(-0.5)));
        opts.replace(AxisOption::XMax(Numeric::new(n as f64 - 0.5)));

        let mut elements = Vec::new();

        for (si, series) in self.series.iter().enumerate() {
            let (meds, q1s, q3s) = &all_stats[si];
            let cn = series.color.to_string();

            // Transparent Q1–Q3 band
            let mut band = Vec::new();
            for (i, q3) in q3s.iter().enumerate() {
                band.push(Coordinate::Plain(i as f64, *q3));
            }
            for (i, q1) in q1s.iter().enumerate().rev() {
                band.push(Coordinate::Plain(i as f64, *q1));
            }
            elements.push(AxisElement::Plot(AddPlot {
                opts: vec![cn.clone(), "opacity=0.3".into(), "draw=none".into(), "forget plot".into()],
                coords: band,
                closed_cycle: true,
            }));

            // Median line
            let line: Vec<Coordinate> = meds.iter().enumerate()
                .map(|(i, median)| Coordinate::Plain(i as f64, *median))
                .collect();
            elements.push(AxisElement::Plot(AddPlot {
                opts: vec![
                    cn,
                    "mark=*".into(),
                    "mark size=2pt".into(),
                    "mark options={solid,draw=white}".into(),
                    "line width=1pt".into(),
                ],
                coords: line,
                closed_cycle: false,
            }));

            elements.push(AxisElement::LegendEntry(series.label.clone()));
        }

        Axis { opts, elements }
    }
}
