use energy_plots::prelude::*;

// ── Helpers ───────────────────────────────────────────────────────────────

fn test_csv() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test_data.csv")
}

fn load_1thread() -> DataFrame {
    DataFrame::from_csv(test_csv())
        .expect("failed to load test CSV")
        .filter(|r| r["threads"] == 1.0)
        .with_column("gflop_j", |r| r["insns"] / r["rapl"] / 1e9)
        .with_column("gflop_s", |r| r["insns"] / r["runtime"] / 1e9)
        .with_column("ipc", |r| r["insns"] / r["cycs"])
}

// ── DataFrame tests ───────────────────────────────────────────────────────

#[test]
fn dataframe_loads_csv() {
    let df = DataFrame::from_csv(test_csv()).expect("load failed");
    assert_eq!(df.len(), 18, "expected 18 data rows");
}

#[test]
fn dataframe_filter() {
    let df = DataFrame::from_csv(test_csv())
        .unwrap()
        .filter(|r| r["threads"] == 1.0);
    assert_eq!(df.len(), 9, "expected 9 rows for 1 thread");
}

#[test]
fn dataframe_with_column() {
    let df = load_1thread();
    let ipc_vals = df.col("ipc");
    assert!(ipc_vals.iter().all(|&v| v > 0.0), "IPC should be positive");
    // IPC ~ insns/cycs, with insns ≈ 125M and cycs ≈ 115M → ~1.09
    assert!(ipc_vals.iter().all(|&v| v > 1.0 && v < 2.0), "IPC should be ~1.0–2.0");
}

#[test]
fn grouped_frame_keys_sorted() {
    let df = load_1thread();
    let grouped = df.group_by("powercap");
    let keys = grouped.keys();
    assert_eq!(keys, &[25.0, 50.0, 75.0]);
}

#[test]
fn grouped_frame_group_values() {
    let df = load_1thread();
    let grouped = df.group_by("powercap");
    // Group 0 = powercap 25.0 → 3 rows
    let vals = grouped.group_values(0, "runtime");
    assert_eq!(vals.len(), 3);
}

// ── Statistics tests ──────────────────────────────────────────────────────

#[test]
fn stats_median() {
    assert_eq!(median(&[3.0, 1.0, 2.0]), 2.0);
    assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), 2.5);
}

#[test]
fn stats_q1_q3() {
    let v = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    assert_eq!(q1(&v), 2.5);
    assert_eq!(q3(&v), 6.5);
}

// ── LinePlot tests ────────────────────────────────────────────────────────

#[test]
fn line_plot_renders_tikzpicture() {
    let df = load_1thread();
    let grouped = df.group_by("powercap");
    let tikz = LinePlot::new(grouped)
        .series("ipc", palette::BLUE, "IPC")
        .xlabel(r"Power limit (\si{\watt})")
        .ylabel("IPC")
        .render();

    assert!(tikz.contains("\\begin{tikzpicture}"), "missing tikzpicture begin");
    assert!(tikz.contains("\\end{tikzpicture}"), "missing tikzpicture end");
    assert!(tikz.contains("\\begin{axis}"), "missing axis begin");
    assert!(tikz.contains("\\end{axis}"), "missing axis end");
    assert!(tikz.contains("common/line"), "missing common/line style");
    assert!(tikz.contains("\\addplot"), "missing addplot");
    assert!(tikz.contains("\\addlegendentry{IPC}"), "missing legend entry");
    assert!(tikz.contains("\\closedcycle"), "missing IQR band closedcycle");
    assert!(tikz.contains(r"Power limit (\si{\watt})"), "missing xlabel");
}

#[test]
fn line_plot_has_three_x_ticks() {
    let df = load_1thread();
    let grouped = df.group_by("powercap");
    let tikz = LinePlot::new(grouped)
        .series("ipc", palette::BLUE, "IPC")
        .render();
    // powercap groups: 25, 50, 75 → indices 0, 1, 2
    assert!(tikz.contains("xtick={0,1,2}"), "expected xtick for 3 groups");
    assert!(tikz.contains("xticklabels={25,50,75}"), "expected xticklabels for powercap values");
}

#[test]
fn line_plot_custom_xtick_labels() {
    let df = load_1thread();
    let grouped = df.group_by("powercap");
    let tikz = LinePlot::new(grouped)
        .series("ipc", palette::BLUE, "IPC")
        .xtick_labels(vec!["low", "mid", "high"])
        .render();
    assert!(tikz.contains("xticklabels={low,mid,high}"), "expected custom xticklabels");
}

// ── BarLinePlot tests ─────────────────────────────────────────────────────

#[test]
fn bar_line_plot_renders() {
    let df = load_1thread();
    let grouped = df.group_by("powercap");
    let tikz = BarLinePlot::new(grouped)
        .bar("gflop_j", palette::GREEN, r"\si{\giga\flop\per\joule}")
        .line("gflop_s", palette::RED, r"\si{\giga\flop\per\second}")
        .xlabel(r"Power limit (\si{\watt})")
        .render();

    assert!(tikz.contains("\\begin{tikzpicture}"));
    assert!(tikz.contains("common/twin-main"), "missing twin-main style");
    assert!(tikz.contains("common/twin,"), "missing twin style");
    assert!(tikz.contains("common/bar"), "missing bar style");
    assert!(tikz.contains("common/line"), "missing line style");
    assert!(tikz.contains("axis y line=right"), "missing right y-axis");
    assert!(tikz.contains("ybar"), "missing ybar option");
    assert!(tikz.contains("\\definecolor{epBar}"), "missing bar color definition");
    assert!(tikz.contains("\\definecolor{epLine}"), "missing line color definition");
    // Both axes must use the dynamically computed right-side padding.
    assert!(
        tikz.contains("width={\\dimexpr \\linewidth - \\epRpad\\relax}"),
        "missing dynamic width expression"
    );
    // The padding setup must include a compile-time font measurement.
    assert!(tikz.contains("\\settowidth{\\epRpad}"), "missing \\settowidth for tick label");
    assert!(tikz.contains("\\settoheight{\\epRlabelH}"), "missing \\settoheight for ylabel");
}

#[test]
fn bar_line_plot_has_iqr_whiskers() {
    let df = load_1thread();
    let grouped = df.group_by("powercap");
    let tikz = BarLinePlot::new(grouped)
        .bar("gflop_j", palette::GREEN, r"\si{\giga\flop\per\joule}")
        .line("gflop_s", palette::RED, r"\si{\giga\flop\per\second}")
        .render();

    // IQR whiskers are emitted as \draw commands on the left axis.
    assert!(tikz.contains("\\draw[black!60"), "missing IQR whisker draw");
    // IQR band on the right axis uses \closedcycle.
    assert!(tikz.contains("\\closedcycle"), "missing IQR band on line axis");
}

#[test]
fn bar_line_plot_legend_entries() {
    let df = load_1thread();
    let grouped = df.group_by("powercap");
    let tikz = BarLinePlot::new(grouped)
        .bar("gflop_j", palette::GREEN, r"\si{\giga\flop\per\joule}")
        .line("gflop_s", palette::RED, r"\si{\giga\flop\per\second}")
        .render();

    // Bar legend on left axis.
    assert!(
        tikz.contains(r"\addlegendentry{\si{\giga\flop\per\joule}}"),
        "missing bar legend entry"
    );
    // Line legend placeholder on left axis, actual series on right.
    assert!(
        tikz.contains(r"\addlegendentry{\si{\giga\flop\per\second}}"),
        "missing line legend entry"
    );
}

#[test]
fn bar_line_plot_width_adapts_to_data_magnitude() {
    // Small values: gflop_s ≈ 0.4 → tick estimate is a short string.
    let tikz_small = BarLinePlot::new(load_1thread().group_by("powercap"))
        .bar("gflop_j", palette::GREEN, r"\si{\giga\flop\per\joule}")
        .line("gflop_s", palette::RED, r"\si{\giga\flop\per\second}")
        .render();

    // Large values: multiply to simulate a large-number axis (e.g. 10 000×).
    let df_large = DataFrame::from_csv(test_csv())
        .unwrap()
        .filter(|r| r["threads"] == 1.0)
        .with_column("gflop_j", |r| r["insns"] / r["rapl"] / 1e9)
        .with_column("big_line", |r| r["insns"] / r["runtime"] / 1e9 * 10_000.0);
    let tikz_large = BarLinePlot::new(df_large.group_by("powercap"))
        .bar("gflop_j", palette::GREEN, r"\si{\giga\flop\per\joule}")
        .line("big_line", palette::RED, "Big label")
        .render();

    // Extract the sample string passed to \settowidth in both outputs.
    fn extract_settowidth_arg(tikz: &str) -> &str {
        let marker = r"\settowidth{\epRpad}{\scriptsize ";
        let start = tikz.find(marker).expect("missing settowidth") + marker.len();
        let end = tikz[start..].find('}').expect("missing closing brace") + start;
        &tikz[start..end]
    }

    let small_arg = extract_settowidth_arg(&tikz_small);
    let large_arg = extract_settowidth_arg(&tikz_large);

    assert!(
        large_arg.len() > small_arg.len(),
        "large-value tick estimate '{large_arg}' should be longer than small-value '{small_arg}'"
    );
}

// ── ZPlot tests ───────────────────────────────────────────────────────────

#[test]
fn zplot_renders() {
    let df = DataFrame::from_csv(test_csv())
        .unwrap()
        .with_column("gflop_j", |r| r["insns"] / r["rapl"] / 1e9)
        .with_column("gflop_s", |r| r["insns"] / r["runtime"] / 1e9);

    let grouped = df.group_by("threads");
    let tikz = ZPlot::new(grouped)
        .x_col("gflop_s")
        .y_col("gflop_j")
        .xlabel(r"\si{\giga\flop\per\second}")
        .ylabel(r"\si{\giga\flop\per\joule}")
        .label_fn(|tc| {
            let n = tc as u32;
            if n == 1 { "1 thread".to_owned() } else { format!("{n} threads") }
        })
        .render();

    assert!(tikz.contains("\\begin{tikzpicture}"));
    assert!(tikz.contains("\\begin{axis}"));
    assert!(tikz.contains("\\addlegendentry{1 thread}"), "missing 1-thread legend");
    assert!(tikz.contains("\\addlegendentry{4 threads}"), "missing 4-thread legend");
    assert!(tikz.contains("epSeries0"), "missing series 0 color");
    assert!(tikz.contains("epSeries1"), "missing series 1 color");
}

// ── Color tests ───────────────────────────────────────────────────────────

#[test]
fn color_hex_parses_green() {
    let c = Color::hex("#3AA640");
    assert_eq!(c.r, 58);
    assert_eq!(c.g, 166);
    assert_eq!(c.b, 64);
}

#[test]
fn color_define_output() {
    let c = Color::rgb(58, 166, 64);
    let def = c.define("epGreen");
    assert_eq!(def, "\\definecolor{epGreen}{RGB}{58,166,64}");
}
