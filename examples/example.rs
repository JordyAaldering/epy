use std::fs;

use epy::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct Record {
    #[serde(skip)]
    _i_also_use_name_columns_sometimes: String,
    threads: usize,
    #[serde(deserialize_with = "from_micro")]
    powercap: f64,
    insns: usize,
    rapl: f64,
    runtime: f64,
    cycs: usize,
}

fn from_micro<'de, D>(de: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let x = usize::deserialize(de)?;
    Ok(x as f64 / 1e6)
}

impl Record {
    fn gflop_s(&self) -> f64 {
        self.insns as f64 / self.runtime / 1e9
    }

    fn gflop_j(&self) -> f64 {
        self.insns as f64 / self.rapl / 1e9
    }

    fn ipc(&self) -> f64 {
        self.insns as f64 / self.cycs as f64
    }
}

fn max_threads(df: &DataFrame<Record>) -> usize {
    df.rows()
        .iter()
        .map(|r| r.threads)
        .max()
        .unwrap()
}

fn twin(df: &DataFrame<Record>) {
    let max_t = max_threads(df);
    let filtered = df.clone().filter(|r| r.threads == max_t);

    let (mut ax0, mut ax1) = TwinPlot::new(
            filtered,
            |r| r.powercap,
            "Power limit (W)",
            "GFLOP/J",
            "GFLOP/s",
        )
        .ax0_bar(|r| r.gflop_j(), "GFLOP/J", "energycolor")
        .ax1_line(|r| r.gflop_s(), "GFLOP/s", "runtimecolor")
        .ax1_line(|r| r.gflop_s() / 2.0, "half", "cyan")
        .build_axes();

    let style = &mut ax0.style;
    style.ymax = Some(0.09);
    style.legend_pos = Some(Anchor::SouthEast);
    filter_xticks_stride(style, 0, 2);
    format_xticks_precision(style, 1, false);
    let style = &mut ax1.style;
    style.ymax = Some(0.6);

    let doc = TikzPicture::from_twin(ax0, ax1);
    doc.write(".build/example_twin.tex").unwrap();
}

fn ipc(df: &DataFrame<Record>) {
    let max_t = max_threads(df);
    let filtered = df.clone().filter(|r| r.threads == max_t);

    let mut ax0 = LinePlot::new(filtered, |r| r.powercap, "Power limit (W)", "IPC")
        .series(|r| r.ipc(), "IPC", "runtimecolor")
        .build_axis()
        .label(Coordinate::AxisCs(5.0, 0.35), "Hello, world!", None)
        .area(Coordinate::AxisCs(5.5, 0.0), Coordinate::AxisCs(12.5, 1.0), "purple!30");
    let style = &mut ax0.style;
    filter_xticks_stride(style, 0, 2);
    format_xticks_precision(style, 1, false);

    // .annot_area((5.5, 0.0), (12.5, 1.0), "black!30".into())
    let doc = TikzPicture::from_axis(ax0);
    doc.write(".build/example_ipc.tex").unwrap();
}

fn power(df: &DataFrame<Record>) {
    let ax = LinePlot::new(
        df.clone(),
        |r| r.powercap,
        "Configured power limit (W)",
        "Actual power draw (W)",
    )
        .grouped_series(|r| r.threads as f64, |r| r.rapl / r.runtime)
        .build_axis();

    let doc = TikzPicture::from_axis(ax);
    doc.write(".build/example_power.tex").unwrap();
}

fn zplot(df: &DataFrame<Record>) {
    let mut ax = ZPlot::new(
            df.clone(),
            |r| r.threads as f64,
            |r| r.powercap,
            |r| r.gflop_s(),
            |r| r.gflop_j(),
            "GFLOP/s",
            "GFLOP/J",
        )
        .build_axis();
    let style = &mut ax.style;
    style.xmin = Some(0.0);
    style.ymin = Some(0.0);

    let doc = TikzPicture::from_axis(ax);
    doc.write(".build/example_zplot.tex").unwrap();
}

#[allow(dead_code)]
#[derive(Deserialize, Clone)]
struct TimeseriesRecord {
    kernel: String,
    size: usize,
    threads: usize,
    powercap: usize,
    runtime: f64,
    energy: f64,
}

fn timeseries() {
    let df: DataFrame<TimeseriesRecord> = DataFrame::from_csv("test_data_timeseries.csv").unwrap();

    let axis = TimeSeries::new(df, "Iteration", "Energy consumption")
        .series(|r| r.energy, "Energy", "energycolor")
        .series(|r| r.runtime, "Runtime", "runtimecolor")
        .build_axis();

    let doc = TikzPicture::from_axis(axis);
    doc.write(".build/example_timeseries.tex").unwrap();
}

fn main() {
    fs::create_dir_all(".build").unwrap();
    let df = DataFrame::from_csv("test_data.csv").unwrap();
    twin(&df);
    ipc(&df);
    power(&df);
    zplot(&df);
    timeseries();
}
