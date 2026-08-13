use std::fs;

use epy::prelude::*;
use ordered_float::OrderedFloat;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct Record {
    #[serde(skip)]
    _name: String,
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
    df.fold(0, |a, b| a.max(b.threads))
}

fn twin(df: &DataFrame<Record>) {
    let max_t = max_threads(df);
    let filtered = df.clone().filter(|_, r| r.threads == max_t);

    let (mut ax0, mut ax1) = TwinPlot::<Record, _>::new(
            |r| OrderedFloat(r.powercap),
            "Power limit (W)",
            "GFLOP/J",
            "GFLOP/s",
        )
        .ax0_bar(&filtered,
            |r| r.gflop_j(),
            AggregationMode::Quartiles,
            "GFLOP/J",
            "energycolor",
        )
        .ax1_line(&filtered,
            |r| r.gflop_s(),
            AggregationMode::Quartiles,
            "GFLOP/s",
            "runtimecolor",
        )
        .ax1_line(&filtered,
            |r| r.gflop_s() / 2.0,
            AggregationMode::Quartiles,
            "half",
            "cyan",
        )
        .build_axes();

    ax0.style.ymax = Some(0.09);
    ax0.style.legend_pos = Some(Anchor::SouthEast);
    ax0.style.filter_xticks(|i| filter_every(i, 3));
    ax0.style.format_xticks(|s| format_precision(s, 1, false));
    ax1.style.ymax = Some(0.62);

    let doc = TikzPicture::from_twin(ax0, ax1);
    doc.write(".build/example_twin.tex").unwrap();
}

fn ipc(df: &DataFrame<Record>) {
    let max_t = max_threads(df);
    let filtered = df.clone().filter(|_, r| r.threads == max_t);

    let mut ax = LinePlot::<Record, usize>::new(
            |r| r.powercap,
            "Power limit (W)",
            "IPC",
        )
        .series(
            &filtered,
            |r| r.ipc(),
            AggregationMode::meanstd(),
            "IPC",
            "runtimecolor",
        )
        .build_axis()
        .label(Cs::Axis(5.0, 0.35), "Hello, world!", None)
        .area(Cs::Axis(5.5, 0.0), Cs::Axis(12.5, 1.0), Some("purple!30"));

    ax.style.filter_xticks(|i| filter_stride(i, 4, 1));
    ax.style.format_xticks(|s| format_precision(s, 1, false));

    let doc = TikzPicture::from_axis(ax);
    doc.write(".build/example_ipc.tex").unwrap();
}

fn power(df: &DataFrame<Record>) {
    let ax = LinePlot::<Record, _>::new(
            |r| r.powercap,
            "Configured power limit (W)",
            "Actual power draw (W)",
        )
        .grouped_series(df,
            |r| r.threads,
            |r| r.rapl / r.runtime,
            AggregationMode::Quartiles,
        )
        .build_axis();

    let doc = TikzPicture::from_axis(ax);
    doc.write(".build/example_power.tex").unwrap();
}

fn zplot(df: &DataFrame<Record>) {
    let mut ax = ZPlot::new(
            |r: &Record| r.threads,
            |r| OrderedFloat(r.powercap),
            |r| r.gflop_s(),
            |r| r.gflop_j(),
            AggregationMode::Quartiles,
            "GFLOP/s",
            "GFLOP/J",
        )
        .build_axis(df);

    ax.style.title = Some("ZPlot example".to_string());
    ax.style.xmin = Some(0.0);
    ax.style.ymin = Some(0.0);

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
    let df: DataFrame<TimeseriesRecord> = DataFrame::from_csv("test_series.csv").unwrap();

    let ax = TimeSeries::<TimeseriesRecord, usize>::new(
            "Iteration",
            "Energy consumption",
        )
        .series(&df,
            |r| r.energy,
            "Energy",
            "energycolor",
        )
        .series(&df,
            |r| r.runtime,
            "Runtime",
            "runtimecolor",
        )
        .build_axis();

    let doc = TikzPicture::from_axis(ax);
    doc.write(".build/example_timeseries.tex").unwrap();
}

#[derive(Deserialize, Clone)]
struct NeaRecord {
    benchmark: String,
    language: String,
    rapl: f64,
}

fn grouped_bar() {
    let df: DataFrame<NeaRecord> = DataFrame::from_csv("nea.csv").unwrap();

    let ax = BarPlot::<NeaRecord, String, String>::new(
            |r| r.benchmark.clone(),
            "Benchmark",
            "Energy",
        )
        .grouped_series(&df,
            |r| r.language.clone(),
            |r| r.rapl,
            AggregationMode::Quartiles,
        )
        .build_axis();

    let doc = TikzPicture::from_axis(ax);
    doc.write(".build/example_nea.tex").unwrap();
}

fn main() {
    fs::create_dir_all(".build").unwrap();
    let df = DataFrame::from_csv("test_data.csv").unwrap();
    twin(&df);
    ipc(&df);
    power(&df);
    zplot(&df);
    timeseries();
    grouped_bar();
}
