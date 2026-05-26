use crate::{data::*, plot::*, tikzir::*};

pub struct TwinPlot<Row> {
    df: DataFrame<Row>,
    group_selector: Box<dyn Fn(&Row) -> f64>,
    ax0_series: Vec<TwinSeries<Row>>,
    ax1_series: Vec<TwinSeries<Row>>,
    ax0_yaxis_label: String,
    ax1_yaxis_label: String,
    xaxis_label: String,
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
    pub fn new<G>(
        df: DataFrame<Row>,
        group_selector: G,
        xaxis_label: &str,
        ax0_yaxis_label: &str,
        ax1_yaxis_label: &str,
    ) -> Self
    where
        G: Fn(&Row) -> f64 + 'static,
    {
        TwinPlot {
            df,
            group_selector: Box::new(group_selector),
            ax0_series: Vec::new(),
            ax1_series: Vec::new(),
            ax0_yaxis_label: ax0_yaxis_label.into(),
            ax1_yaxis_label: ax1_yaxis_label.into(),
            xaxis_label: xaxis_label.into(),
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

    pub fn ax1_bar<Y>(
        mut self,
        selector: Y,
        label: &str,
        color: &str,
    ) -> Self
    where
        Y: Fn(&Row) -> f64 + 'static,
    {
        self.ax1_series.push(TwinSeries {
            kind: TwinSeriesKind::Bar,
            selector: Box::new(selector),
            label: label.into(),
            color: color.into(),
        });
        self
    }

    pub fn ax1_line<Y>(
        mut self,
        selector: Y,
        label: &str,
        color: &str,
    ) -> Self
    where
        Y: Fn(&Row) -> f64 + 'static,
    {
        self.ax1_series.push(TwinSeries {
            kind: TwinSeriesKind::Line,
            selector: Box::new(selector),
            label: label.into(),
            color: color.into(),
        });
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

    fn build_left_axis(&self) -> Axis {
        let grouped = self.grouped();
        let keys = grouped.keys().to_vec();
        let n = keys.len();

        let mut options = common_axis_options();
        options.name = Some("mainaxis".into());
        options.width = Some(Dimension::Code("{\\epyfigurewidth-\\epyrpad}".into()));
        options.height = Some(Dimension::Code("{\\epyheightratio*\\epyfigurewidth}".into()));

        options.xlabel = Some(self.xaxis_label.clone());
        options.ylabel = Some(self.ax0_yaxis_label.clone());

        options.xmin = Some(-0.5);
        options.xmax = Some(n as f64 - 0.5);
        if self.ax0_series.iter().any(|s| s.kind == TwinSeriesKind::Bar) {
            options.ymin = Some(0.0);
        }

        options.xticks = (0..n).collect::<Vec<_>>().into();
        options.xtick_labels = Some(keys.iter().map(ToString::to_string).collect::<Vec<_>>().into());
        options.trim_axis_right = true;

        let mut elements = Vec::new();
        self.push_axis_series_elements(&grouped, &self.ax0_series, &mut elements, true, 0);
        let ax0_line_count = self.ax0_series.iter().filter(|s| s.kind == TwinSeriesKind::Line).count();
        self.push_legend_images_for_series(&self.ax1_series, &mut elements, ax0_line_count);

        Axis { style: options, data: elements }
    }

    fn build_right_axis(&self) -> Axis {
        let grouped = self.grouped();
        let n = grouped.num_groups();

        let mut options = common_axis_options();
        options.width = Some(Dimension::Code("{\\epyfigurewidth-\\epyrpad}".into()));

        options.xmin = Some(-0.5);
        options.xmax = Some(n as f64 - 0.5);
        if self.ax1_series.iter().any(|s| s.kind == TwinSeriesKind::Bar) {
            options.ymin = Some(0.0);
        }

        options.ylabel = Some(self.ax1_yaxis_label.clone());
        options.grid = Some(GridLines::None);
        options.xticks = TickPositions::Empty;
        options.xtick_labels = Some(TickLabels::Empty);
        options.ytick_pos = Some(TickPos::Right);
        options.trim_axis_left = true;
        options.axis_x_line = Some(CellAlign::None);
        options.axis_y_line_star = Some(CellAlign::Right);
        options.anchor = Some(Anchor::SouthWest);
        options.at = Some("(mainaxis.south west)".into());

        let mut elements = Vec::new();
        let ax0_line_count = self.ax0_series.iter().filter(|s| s.kind == TwinSeriesKind::Line).count();
        self.push_axis_series_elements(&grouped, &self.ax1_series, &mut elements, false, ax0_line_count);

        Axis { style: options, data: elements }
    }

    fn push_axis_series_elements(
        &self,
        grouped: &GroupedFrame<Row>,
        series: &Vec<TwinSeries<Row>>,
        elements: &mut Vec<AxisElement>,
        include_in_legend: bool,
        line_marker_start_index: usize,
    ) {
        let mut line_series_count = 0;
        for spec in series {
            let quartiles = grouped.quartiles_by_group(&spec.selector);
            let color = spec.color.clone();

            match spec.kind {
                TwinSeriesKind::Bar => {
                    let bar_coords: Vec<Coordinate> = quartiles.medians.iter().enumerate()
                        .map(|(i, median)| Coordinate::Plain(i as f64, *median))
                        .collect();

                    let plot_options = StyleBuilder::default()
                        .ybar(true)
                        .bar_width(0.7)
                        .fill(color.clone())
                        .draw("none")
                        .area_legend(true)
                        .forget_plot(!include_in_legend)
                        .build()
                        .unwrap();

                    elements.push(AxisElement::AddPlot {
                        style: plot_options,
                        coordinates: bar_coords,
                        closed_cycle: false,
                    });
                    if include_in_legend {
                        elements.push(AxisElement::LegendEntry(spec.label.clone()));
                    }

                    for i in 0..quartiles.q1s.len() {
                        let options = StyleBuilder::default()
                            .color("black!90")
                            .line_width(Dimension::Pt(0.9))
                            .build()
                            .unwrap();
                        elements.push(AxisElement::Draw {
                            style: options,
                            from: Coordinate::AxisCs(i as f64, quartiles.q1s[i]),
                            to: Coordinate::AxisCs(i as f64, quartiles.q3s[i]),
                        });
                    }
                }
                TwinSeriesKind::Line => {
                    let mut band = Vec::new();
                    for (i, q3) in quartiles.q3s.iter().enumerate() {
                        band.push(Coordinate::Plain(i as f64, *q3));
                    }
                    for (i, q1) in quartiles.q1s.iter().enumerate().rev() {
                        band.push(Coordinate::Plain(i as f64, *q1));
                    }

                    let err_options = StyleBuilder::default()
                        .draw("none")
                        .fill(color.clone())
                        .fill_opacity(0.3)
                        .forget_plot(true)
                        .build()
                        .unwrap();

                    elements.push(AxisElement::AddPlot {
                        style: err_options,
                        coordinates: band,
                        closed_cycle: true,
                    });

                    let line: Vec<Coordinate> = quartiles.medians.iter().enumerate()
                        .map(|(i, median)| Coordinate::Plain(i as f64, *median))
                        .collect();
                    let marker_index = (line_marker_start_index + line_series_count) % MARKERS.len();

                    let plot_options = StyleBuilder::default()
                        .color(color.clone())
                        .line_width(Dimension::Pt(1.0))
                        .mark(MARKERS[marker_index].to_string())
                        .mark_size(Dimension::Pt(MARK_SIZE_PT))
                        .mark_options(StyleBuilder::default()
                            .solid(true)
                            .draw("white".to_string())
                            .line_width(Dimension::Pt(-MARK_OUTLINE_PT))
                            .build()
                            .unwrap()
                        )
                        .forget_plot(!include_in_legend)
                        .build()
                        .unwrap();

                    elements.push(AxisElement::AddPlot {
                        style: plot_options,
                        coordinates: line,
                        closed_cycle: false,
                    });

                    if include_in_legend {
                        elements.push(AxisElement::LegendEntry(spec.label.clone()));
                    }

                    line_series_count += 1;
                }
            }
        }
    }

    fn push_legend_images_for_series(
        &self,
        series: &[TwinSeries<Row>],
        elements: &mut Vec<AxisElement>,
        line_marker_start_index: usize,
    ) {
        let mut line_series_count = 0;
        for spec in series {
            match spec.kind {
                TwinSeriesKind::Bar => {
                    let legend_style = StyleBuilder::default()
                        .ybar(true)
                        .bar_width(0.7)
                        .fill(spec.color.clone())
                        .draw("none")
                        .area_legend(true)
                        .build()
                        .unwrap();
                    elements.push(AxisElement::LegendImage(legend_style));
                }
                TwinSeriesKind::Line => {
                    let marker_index = (line_marker_start_index + line_series_count) % MARKERS.len();
                    let legend_style = StyleBuilder::default()
                        .fill(spec.color.clone())
                        .line_width(Dimension::Pt(1.0))
                        .mark(MARKERS[marker_index].to_string())
                        .mark_size(Dimension::Pt(MARK_SIZE_PT))
                        .mark_options(StyleBuilder::default()
                            .solid(true)
                            .draw("white".to_string())
                            .line_width(Dimension::Pt(-MARK_OUTLINE_PT))
                            .build()
                            .unwrap()
                        )
                        .build()
                        .unwrap();
                    elements.push(AxisElement::LegendImage(legend_style));
                    line_series_count += 1;
                }
            }
            elements.push(AxisElement::LegendEntry(spec.label.clone()));
        }
    }
}
