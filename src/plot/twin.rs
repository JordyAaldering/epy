use crate::{
    data::{DataFrame, GroupedFrame},
    ir::*,
    plot::{common_axis_options, quartiles},
};

pub struct TwinPlot<Row> {
    df: DataFrame<Row>,
    group_selector: Box<dyn Fn(&Row) -> f64>,
    ax0_series: Vec<TwinSeries<Row>>,
    ax1_series: Vec<TwinSeries<Row>>,
    ax0_yaxis_label: String,
    ax1_yaxis_label: String,
    xaxis_label: String,
    xtick_labels: Option<Vec<String>>,
}

struct TwinSeries<Row> {
    kind: TwinSeriesKind,
    selector: Box<dyn Fn(&Row) -> f64>,
    label: String,
    color: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TwinSeriesKind {
    Bar,
    Line,
}

impl<Row: Clone> TwinPlot<Row> {
    pub fn new(
        df: DataFrame<Row>,
        group_selector: impl Fn(&Row) -> f64 + 'static,
        xaxis_label: &str,
        ax0_yaxis_label: &str,
        ax1_yaxis_label: &str,
    ) -> Self {
        TwinPlot {
            df,
            group_selector: Box::new(group_selector),
            ax0_series: Vec::new(),
            ax1_series: Vec::new(),
            ax0_yaxis_label: ax0_yaxis_label.into(),
            ax1_yaxis_label: ax1_yaxis_label.into(),
            xaxis_label: xaxis_label.into(),
            xtick_labels: None,
        }
    }

    pub fn ax0_bar(
        mut self,
        selector: impl Fn(&Row) -> f64 + 'static,
        label: &str,
        color: &str,
    ) -> Self {
        self.ax0_series.push(TwinSeries {
            kind: TwinSeriesKind::Bar,
            selector: Box::new(selector),
            label: label.into(),
            color: color.into(),
        });
        self
    }

    pub fn ax0_line(
        mut self,
        selector: impl Fn(&Row) -> f64 + 'static,
        label: &str,
        color: &str,
    ) -> Self {
        self.ax0_series.push(TwinSeries {
            kind: TwinSeriesKind::Line,
            selector: Box::new(selector),
            label: label.into(),
            color: color.into(),
        });
        self
    }

    pub fn ax1_bar(
        mut self,
        selector: impl Fn(&Row) -> f64 + 'static,
        label: &str,
        color: &str,
    ) -> Self {
        self.ax1_series.push(TwinSeries {
            kind: TwinSeriesKind::Bar,
            selector: Box::new(selector),
            label: label.into(),
            color: color.into(),
        });
        self
    }

    pub fn ax1_line(
        mut self,
        selector: impl Fn(&Row) -> f64 + 'static,
        label: &str,
        color: &str,
    ) -> Self {
        self.ax1_series.push(TwinSeries {
            kind: TwinSeriesKind::Line,
            selector: Box::new(selector),
            label: label.into(),
            color: color.into(),
        });
        self
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
        let keys = grouped.keys().to_vec();
        let n = keys.len();

        let mut opts = common_axis_options();
        opts.replace(AxisOption::Name("mainaxis".into()));
        opts.replace(AxisOption::TrimAxisRight);
        opts.replace(AxisOption::Width("{\\epyfigurewidth-\\epyrpad}".into()));
        opts.replace(AxisOption::Height("{\\epyheightratio*\\epyfigurewidth}".into()));
        opts.replace(AxisOption::XLabel(self.xaxis_label.clone()));
        opts.replace(AxisOption::YLabel(self.ax0_yaxis_label.clone()));
        if self.ax0_series.iter().any(|s| s.kind == TwinSeriesKind::Bar) {
            opts.replace(AxisOption::YMin(Numeric::new(0.0)));
        }
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
        self.push_axis_series_elements(&grouped, &self.ax0_series, &mut elements, true);
        self.push_legend_images_for_series(&self.ax1_series, &mut elements);

        Axis { opts, elements }
    }

    fn build_right_axis(&self) -> Axis {
        let grouped = self.grouped();
        let n = grouped.num_groups();

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
        opts.replace(AxisOption::YLabel(self.ax1_yaxis_label.clone()));
        if self.ax1_series.iter().any(|s| s.kind == TwinSeriesKind::Bar) {
            opts.replace(AxisOption::YMin(Numeric::new(0.0)));
        }
        opts.replace(AxisOption::Width("{\\epyfigurewidth-\\epyrpad}".into()));
        opts.replace(AxisOption::Height("{\\epyheightratio*\\epyfigurewidth}".into()));
        opts.replace(AxisOption::XMin(Numeric::new(-0.5)));
        opts.replace(AxisOption::XMax(Numeric::new(n as f64 - 0.5)));

        let mut elements = Vec::new();
        self.push_axis_series_elements(&grouped, &self.ax1_series, &mut elements, false);

        Axis { opts, elements }
    }

    fn push_axis_series_elements(
        &self,
        grouped: &GroupedFrame<Row>,
        series: &[TwinSeries<Row>],
        elements: &mut Vec<AxisElement>,
        include_legend: bool,
    ) {
        for (series_i, spec) in series.iter().enumerate() {
            let (_, meds, q1s, q3s) = self.stats_for_selector(grouped, &*spec.selector);
            let color = spec.color.clone();

            match spec.kind {
                TwinSeriesKind::Bar => {
                    let bar_coords: Vec<Coordinate> = meds.iter().enumerate()
                        .map(|(i, median)| Coordinate::Plain(i as f64, *median))
                        .collect();
                    let mut plot_opts = vec![
                        "ybar".into(),
                        "bar width=0.7".into(),
                        format!("fill={}", color),
                        "draw=none".into(),
                        "area legend".into(),
                    ];
                    if !include_legend {
                        plot_opts.push("forget plot".into());
                    }
                    elements.push(AxisElement::Plot(AddPlot {
                        opts: plot_opts,
                        coords: bar_coords,
                        closed_cycle: false,
                    }));
                    if include_legend {
                        elements.push(AxisElement::LegendEntry(spec.label.clone()));
                    }

                    for i in 0..q1s.len() {
                        elements.push(AxisElement::DrawLine {
                            options: vec!["black!90".into(), "line width=0.9pt".into()],
                            from: Coordinate::AxisCs(i as f64, q1s[i]),
                            to: Coordinate::AxisCs(i as f64, q3s[i]),
                        });
                    }
                }
                TwinSeriesKind::Line => {
                    let mut band = Vec::new();
                    for (i, q3) in q3s.iter().enumerate() {
                        band.push(Coordinate::Plain(i as f64, *q3));
                    }
                    for (i, q1) in q1s.iter().enumerate().rev() {
                        band.push(Coordinate::Plain(i as f64, *q1));
                    }
                    elements.push(AxisElement::Plot(AddPlot {
                        opts: vec![
                            format!("fill={}", color),
                            "fill opacity=0.3".into(),
                            "draw=none".into(),
                            "forget plot".into(),
                        ],
                        coords: band,
                        closed_cycle: true,
                    }));

                    let line: Vec<Coordinate> = meds.iter().enumerate()
                        .map(|(i, median)| Coordinate::Plain(i as f64, *median))
                        .collect();
                    let mut plot_opts = vec![
                        color,
                        "line width=1pt".into(),
                        format!("mark={}", MARKERS[series_i % MARKERS.len()]),
                        format!("mark size={}pt", MARK_SIZE_PT),
                        format!("mark options={{solid,draw=white,line width=-{}pt}}", MARK_OUTLINE_PT),
                    ];
                    if !include_legend {
                        plot_opts.push("forget plot".into());
                    }
                    elements.push(AxisElement::Plot(AddPlot {
                        opts: plot_opts,
                        coords: line,
                        closed_cycle: false,
                    }));
                    if include_legend {
                        elements.push(AxisElement::LegendEntry(spec.label.clone()));
                    }
                }
            }
        }
    }

    fn push_legend_images_for_series(
        &self,
        series: &[TwinSeries<Row>],
        elements: &mut Vec<AxisElement>,
    ) {
        for (series_i, spec) in series.iter().enumerate() {
            match spec.kind {
                TwinSeriesKind::Bar => {
                    elements.push(AxisElement::LegendImage(vec![
                        "ybar".into(),
                        "bar width=0.7".into(),
                        format!("fill={}", spec.color),
                        "draw=none".into(),
                        "area legend".into(),
                    ]));
                }
                TwinSeriesKind::Line => {
                    elements.push(AxisElement::LegendImage(vec![
                        spec.color.clone(),
                        "line width=1pt".into(),
                        format!("mark={}", MARKERS[series_i % MARKERS.len()]),
                        format!("mark size={}pt", MARK_SIZE_PT),
                        format!("mark options={{solid,draw=white,line width=-{}pt}}", MARK_OUTLINE_PT),
                    ]));
                }
            }
            elements.push(AxisElement::LegendEntry(spec.label.clone()));
        }
    }
}
