use crate::{
    data::{DataFrame, GroupedFrame},
    ir::*,
    plot::{common_axis_options, quartiles},
};

pub struct TwinPlot<Row> {
    df: DataFrame<Row>,
    group_selector: Box<dyn Fn(&Row) -> f64>,
    bar_selector: Box<dyn Fn(&Row) -> f64>,
    bar_label: String,
    line_selector: Box<dyn Fn(&Row) -> f64>,
    line_label: String,
    xaxis_label: String,
    xtick_labels: Option<Vec<String>>,
}

impl<Row: Clone> TwinPlot<Row> {
    pub fn new(
        df: DataFrame<Row>,
        group_selector: impl Fn(&Row) -> f64 + 'static,
        bar_selector: impl Fn(&Row) -> f64 + 'static,
        bar_label: &str,
        line_selector: impl Fn(&Row) -> f64 + 'static,
        line_label: &str,
        xaxis_label: &str,
    ) -> Self {
        TwinPlot {
            df,
            group_selector: Box::new(group_selector),
            bar_selector: Box::new(bar_selector),
            bar_label: bar_label.into(),
            line_selector: Box::new(line_selector),
            line_label: line_label.into(),
            xaxis_label: xaxis_label.into(),
            xtick_labels: None,
        }
    }

    pub fn xtick_labels(mut self, labels: Vec<impl Into<String>>) -> Self {
        self.xtick_labels = Some(labels.into_iter().map(|l| l.into()).collect());
        self
    }

    pub fn build_axes(&self) -> (Axis, Axis) {
        let ax1 = self.build_left_axis();
        let ax2 = self.build_right_axis();
        (ax1, ax2)
    }

    fn grouped(&self) -> GroupedFrame<Row> {
        self.df.clone().group_by(|row| (self.group_selector)(row))
    }

    fn stats_for_selector(
        &self,
        grouped: &GroupedFrame<Row>,
        selector: &dyn Fn(&Row) -> f64,
    ) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
        let keys = grouped.keys().to_vec();
        let mut meds = Vec::with_capacity(grouped.num_groups());
        let mut q1s = Vec::with_capacity(grouped.num_groups());
        let mut q3s = Vec::with_capacity(grouped.num_groups());

        for gi in 0..grouped.num_groups() {
            let vals = grouped.group_values(gi, selector);
            let qs = quartiles(&vals);
            meds.push(qs.median);
            q1s.push(qs.q1);
            q3s.push(qs.q3);
        }

        (keys, meds, q1s, q3s)
    }

    fn build_left_axis(&self) -> Axis {
        let grouped = self.grouped();
        let (keys, meds, q1s, q3s) = self.stats_for_selector(&grouped, &*self.bar_selector);
        let n = keys.len();

        let mut opts = common_axis_options();
        opts.replace(AxisOption::Name("mainaxis".into()));
        opts.replace(AxisOption::TrimAxisRight);
        opts.replace(AxisOption::Width("{\\dimexpr\\epyfigurewidth-\\epyrpad\\relax}".into()));
        opts.replace(AxisOption::Height("{\\dimexpr\\epyheightratio\\epyfigurewidth\\relax}".into()));
        opts.replace(AxisOption::XLabel(self.xaxis_label.clone()));
        opts.replace(AxisOption::YLabel(self.bar_label.clone()));
        opts.replace(AxisOption::YMin(Numeric::new(0.0)));
        opts.replace(AxisOption::XMin(Numeric::new(-0.5)));
        opts.replace(AxisOption::XMax(Numeric::new(n as f64 - 0.5)));
        opts.replace(AxisOption::XTicks((0..n).map(|i| i.to_string()).collect()));
        opts.replace(AxisOption::XTickLabels(
            if let Some(ref lbls) = self.xtick_labels {
                lbls.clone()
            } else {
                keys.iter().map(ToString::to_string).collect()
            }
        ));

        let mut elements = Vec::new();

        // Filled bars (median height)
        let bar_coords: Vec<Coordinate> = meds.iter().enumerate()
            .map(|(i, median)| Coordinate::Plain(i as f64, *median))
            .collect();
        elements.push(AxisElement::Plot(AddPlot {
            opts: vec![
                "ybar".into(),
                "bar width=0.7".into(),
                "fill=epyenergycolor".into(),
                "draw=none".into(),
                "area legend".into(),
            ],
            coords: bar_coords,
            closed_cycle: false,
        }));
        elements.push(AxisElement::LegendEntry(self.bar_label.clone()));

        // Error whiskers
        for i in 0..q1s.len() {
            elements.push(AxisElement::DrawLine {
                options: vec!["black!90".into(), "line width=0.9pt".into()],
                from: Coordinate::AxisCs(i as f64, q1s[i]),
                to: Coordinate::AxisCs(i as f64, q3s[i]),
            });
        }

        // Legend image + entry for the right-axis line series
        elements.push(AxisElement::LegendImage(vec![
            "epyruntimecolor".into(),
            "mark=*".into(),
            "mark options={solid,draw=white}".into(),
            "mark size=2pt".into(),
            "line width=1pt".into(),
        ]));
        elements.push(AxisElement::LegendEntry(self.line_label.clone()));

        Axis { opts, elements }
    }

    fn build_right_axis(&self) -> Axis {
        let grouped = self.grouped();
        let (_, meds, q1s, q3s) = self.stats_for_selector(&grouped, &*self.line_selector);

        let mut opts = common_axis_options();
        opts.replace(AxisOption::AtMainAxisSouthWest);
        opts.replace(AxisOption::AnchorSouthWest);
        opts.replace(AxisOption::TrimAxisLeft);
        opts.replace(AxisOption::AxisXLineNone);
        opts.replace(AxisOption::XMajorGrids(false));
        opts.replace(AxisOption::YMajorGrids(false));
        opts.replace(AxisOption::EmptyXTicks);
        opts.replace(AxisOption::EmptyXTickLabels);
        opts.replace(AxisOption::AxisYLineRight);
        opts.remove(&AxisOption::YTickPosLeft);
        opts.replace(AxisOption::YTickPosRight);
        opts.replace(AxisOption::Width("{\\dimexpr\\epyfigurewidth-\\epyrpad\\relax}".into()));
        opts.replace(AxisOption::Height("{\\dimexpr\\epyheightratio\\epyfigurewidth\\relax}".into()));
        opts.replace(AxisOption::YLabel(self.line_label.clone()));

        let mut band = Vec::new();
        for (i, q3) in q3s.iter().enumerate() {
            band.push(Coordinate::Plain(i as f64, *q3));
        }
        for (i, q1) in q1s.iter().enumerate().rev() {
            band.push(Coordinate::Plain(i as f64, *q1));
        }

        let line: Vec<Coordinate> = meds.iter().enumerate()
            .map(|(i, median)| Coordinate::Plain(i as f64, *median))
            .collect();

        Axis {
            opts,
            elements: vec![
                AxisElement::Plot(AddPlot {
                    opts: vec!["fill=epyruntimecolor".into(), "fill opacity=0.3".into(), "draw=none".into(), "forget plot".into()],
                    coords: band,
                    closed_cycle: true,
                }),
                AxisElement::Plot(AddPlot {
                    opts: vec![
                        "epyruntimecolor".into(),
                        "mark=*".into(),
                        "mark options={solid,draw=white}".into(),
                        "mark size=2pt".into(),
                        "line width=1pt".into(),
                    ],
                    coords: line,
                    closed_cycle: false,
                }),
            ],
        }
    }
}
