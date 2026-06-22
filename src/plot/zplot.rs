use crate::{data::*, plot::*, tikzir::*};

pub struct ZPlot<Row> {
    series_selector: Box<dyn Fn(&Row) -> f64>,
    hue_selector: Box<dyn Fn(&Row) -> f64>,
    x_selector: Box<dyn Fn(&Row) -> f64>,
    y_selector: Box<dyn Fn(&Row) -> f64>,
    aggregation: AggregationMode,
    agg_filter: Option<Box<dyn Fn(f64, f64, f64) -> bool>>,
    xaxis_label: String,
    yaxis_label: String,
}

impl<Row: Clone> ZPlot<Row> {
    /// Create a scatter plot where each unique value of `series_selector` becomes a
    /// separate series in the legend. Within each series, rows are grouped by
    /// `hue_selector` and the aggregated `(x_selector, y_selector)` point is plotted per group.
    pub fn new<G, H, X, Y>(
        series_selector: G,
        hue_selector: H,
        x_selector: X,
        y_selector: Y,
        aggregation: AggregationMode,
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
            series_selector: Box::new(series_selector),
            hue_selector: Box::new(hue_selector),
            x_selector: Box::new(x_selector),
            y_selector: Box::new(y_selector),
            aggregation,
            agg_filter: None,
            xaxis_label: xaxis_label.into(),
            yaxis_label: yaxis_label.into(),
        }
    }

    /// Filter elements after aggregation.
    pub fn with_filter(mut self, filter: impl Fn(f64, f64, f64) -> bool + 'static) -> Self {
        self.agg_filter = Some(Box::new(filter));
        self
    }

    pub fn build_axis(&self, df: &DataFrame<Row>) -> Axis {
        let grouped = df.clone().group_by(|row| (self.series_selector)(row));
        let mut series_groups: Vec<(f64, Vec<Cs>)> = Vec::new();

        for gi in 0..grouped.num_groups() {
            let series_key = grouped.keys()[gi];
            let subgroup = grouped.subgroup_by(gi, |row| (self.hue_selector)(row));
            let x_summary = subgroup.summarize_by_group(&self.x_selector, self.aggregation);
            let y_summary = subgroup.summarize_by_group(&self.y_selector, self.aggregation);

            let mut points_by_hue: Vec<(f64, f64, f64)> = x_summary
                .keys
                .iter()
                .copied()
                .zip(x_summary.centers.iter().copied())
                .zip(y_summary.centers.iter().copied())
                .map(|((hue, x), y)| (hue, x, y))
                .filter(|(hue, x, y)| {
                    if let Some(filter) = &self.agg_filter {
                        filter(*hue, *x, *y)
                    } else {
                        true
                    }
                })
                .collect();
            points_by_hue.sort_by(|a, b| f64::total_cmp(&a.0, &b.0));

            let coordinates = points_by_hue
                .into_iter()
                .map(|(_, x, y)| Cs::Plain(x, y))
                .collect();
            series_groups.push((series_key, coordinates));
        }

        let mut style = common_axis_style();
        style.xlabel = Some(self.xaxis_label.clone());
        style.ylabel = Some(self.yaxis_label.clone());
        line_legend_modifier(&mut style);
        style.x_major_grids = true;
        style.ymin = Some(0.0);

        let xgrid_style = style.style_overrides.entry("x grid style".into());
        xgrid_style.or_default().color = Some(GRID_COLOR.into());

        let mut elements = Vec::new();
        for (gi, (key, coordinates)) in series_groups.into_iter().enumerate() {
            let plot_options = StyleBuilder::default()
                .color(format!("colorblind{}", gi))
                .line_width(Dimension::Pt(1.0))
                .mark(MARKERS[gi % MARKERS.len()])
                .mark_size(Dimension::Pt(MARK_SIZE_PT))
                .mark_options(StyleBuilder::default()
                    .solid(true)
                    .draw("white")
                    .line_width(Dimension::Pt(-MARK_OUTLINE_PT))
                    .build()
                    .unwrap()
                )
                .build()
                .unwrap();

            elements.push(AxisElement::AddPlot {
                style: plot_options,
                coordinates,
                closed_cycle: false,
            });
            elements.push(AxisElement::LegendEntry(key.to_string()));
        }

        Axis { style, data: elements }
    }
}
