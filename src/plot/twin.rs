use std::hash::Hash;

use crate::{data::*, plot::*, tikzir::*};

pub struct TwinPlot<'a, Row, K> {
    group_selector: Box<dyn Fn(&Row) -> K>,
    ax0_series: Vec<TwinSeries<'a, Row>>,
    ax1_series: Vec<TwinSeries<'a, Row>>,
    ax0_yaxis_label: String,
    ax1_yaxis_label: String,
    xaxis_label: String,
}

struct TwinSeries<'a, Row> {
    df: &'a DataFrame<Row>,
    kind: TwinSeriesKind,
    selector: Box<dyn Fn(&Row) -> f64>,
    aggregation: AggregationMode,
    label: String,
    color: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TwinSeriesKind {
    Bar,
    Line,
}

impl<'a, Row, K> TwinPlot<'a, Row, K>
where
    Row: Clone,
    K: Clone + Eq + Hash + PartialOrd + ToString,
{
    pub fn new<G>(
        group_selector: G,
        xaxis_label: &str,
        ax0_yaxis_label: &str,
        ax1_yaxis_label: &str,
    ) -> Self
    where
        G: Fn(&Row) -> K + 'static,
    {
        TwinPlot {
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
        df: &'a DataFrame<Row>,
        selector: impl Fn(&Row) -> f64 + 'static,
        aggregation: AggregationMode,
        label: &str,
        color: &str,
    ) -> Self {
        self.ax0_series.push(TwinSeries {
            df,
            kind: TwinSeriesKind::Bar,
            selector: Box::new(selector),
            aggregation,
            label: label.into(),
            color: color.into(),
        });
        self
    }

    pub fn ax0_line(
        mut self,
        df: &'a DataFrame<Row>,
        selector: impl Fn(&Row) -> f64 + 'static,
        aggregation: AggregationMode,
        label: &str,
        color: &str,
    ) -> Self {
        self.ax0_series.push(TwinSeries {
            df,
            kind: TwinSeriesKind::Line,
            selector: Box::new(selector),
            aggregation,
            label: label.into(),
            color: color.into(),
        });
        self
    }

    pub fn ax1_bar<Y>(
        mut self,
        df: &'a DataFrame<Row>,
        selector: Y,
        aggregation: AggregationMode,
        label: &str,
        color: &str,
    ) -> Self
    where
        Y: Fn(&Row) -> f64 + 'static,
    {
        self.ax1_series.push(TwinSeries {
            df,
            kind: TwinSeriesKind::Bar,
            selector: Box::new(selector),
            aggregation,
            label: label.into(),
            color: color.into(),
        });
        self
    }

    pub fn ax1_line<Y>(
        mut self,
        df: &'a DataFrame<Row>,
        selector: Y,
        aggregation: AggregationMode,
        label: &str,
        color: &str,
    ) -> Self
    where
        Y: Fn(&Row) -> f64 + 'static,
    {
        self.ax1_series.push(TwinSeries {
            df,
            kind: TwinSeriesKind::Line,
            selector: Box::new(selector),
            aggregation,
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

    fn build_left_axis(&self) -> Axis {
        let keys = self.reference_keys();
        let n = keys.len();

        let mut style = common_axis_style();

        style.xlabel = Some(self.xaxis_label.clone());
        style.ylabel = Some(self.ax0_yaxis_label.clone());
        style.xmin = Some(-0.5);
        style.xmax = Some(n as f64 - 0.5);

        if self.ax0_series.iter().any(|s| s.kind == TwinSeriesKind::Bar) {
            style.ymin = Some(0.0);
        }

        // The first axis handles the legend of both axes, so check both
        if self.ax0_series.iter().any(|s| s.kind == TwinSeriesKind::Bar)
            || self.ax1_series.iter().any(|s| s.kind == TwinSeriesKind::Bar)
        {
            bar_legend_modifier(&mut style);
        }
        if self.ax0_series.iter().any(|s| s.kind == TwinSeriesKind::Line)
            || self.ax1_series.iter().any(|s| s.kind == TwinSeriesKind::Line)
        {
            line_legend_modifier(&mut style);
        }

        style.xticks = (0..n).collect::<Vec<_>>().into();
        style.xtick_labels = Some(keys.iter().map(ToString::to_string).collect::<Vec<_>>().into());

        let mut data = Vec::new();
        self.push_axis_series_elements(&self.ax0_series, &mut data, true, 0);
        let ax0_line_count = self.ax0_series.iter().filter(|s| s.kind == TwinSeriesKind::Line).count();
        self.push_legend_images_for_series(&self.ax1_series, &mut data, ax0_line_count);

        Axis { style, data }
    }

    fn build_right_axis(&self) -> Axis {
        let n = self.reference_keys().len();

        let mut style = common_axis_style();
        style.ylabel = Some(self.ax1_yaxis_label.clone());
        style.xmin = Some(-0.5);
        style.xmax = Some(n as f64 - 0.5);

        if self.ax1_series.iter().any(|s| s.kind == TwinSeriesKind::Bar) {
            style.ymin = Some(0.0);
        }

        let mut data = Vec::new();
        let ax0_line_count = self.ax0_series.iter().filter(|s| s.kind == TwinSeriesKind::Line).count();
        self.push_axis_series_elements(&self.ax1_series, &mut data, false, ax0_line_count);

        Axis { style, data }
    }

    fn reference_keys(&self) -> Vec<K> {
        let spec = self.ax0_series.first().or_else(|| self.ax1_series.first())
            .expect("At least one series must be added to the twin plot");
        spec.df.clone().group_by(|row| (self.group_selector)(row)).keys().to_vec()
    }

    fn push_axis_series_elements(
        &self,
        series: &[TwinSeries<'a, Row>],
        elements: &mut Vec<AxisElement>,
        include_in_legend: bool,
        line_marker_start_index: usize,
    ) {
        let mut line_series_count = 0;
        for spec in series {
            let grouped = spec.df.clone().group_by(|row| (self.group_selector)(row));
            let summary = grouped.summarize_by_group(&spec.selector, spec.aggregation);
            let color = spec.color.clone();

            match spec.kind {
                TwinSeriesKind::Bar => {
                    let bar_coords: Vec<Cs> = summary.centers.iter().enumerate()
                        .map(|(i, center)| Cs::Plain(i as f64, *center))
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

                    for i in 0..summary.lowers.len() {
                        let options = StyleBuilder::default()
                            .color("black!90")
                            .line_width(Dimension::Pt(0.9))
                            .build()
                            .unwrap();
                        elements.push(AxisElement::Draw {
                            style: options,
                            from: Cs::Axis(i as f64, summary.lowers[i]),
                            to: Cs::Axis(i as f64, summary.uppers[i]),
                        });
                    }
                }
                TwinSeriesKind::Line => {
                    let mut band = Vec::new();
                    for (i, upper) in summary.uppers.iter().enumerate() {
                        band.push(Cs::Plain(i as f64, *upper));
                    }
                    for (i, lower) in summary.lowers.iter().enumerate().rev() {
                        band.push(Cs::Plain(i as f64, *lower));
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

                    let line: Vec<Cs> = summary.centers.iter().enumerate()
                        .map(|(i, center)| Cs::Plain(i as f64, *center))
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
        series: &[TwinSeries<'a, Row>],
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
