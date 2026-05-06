# energy-plots

A small Rust DSL for generating clean [PGFPlots](https://ctan.org/pkg/pgfplots) /
TikZ code from CSV energy-measurement data.

---

## Why?

The Python `matplot2tikz` library produces verbose, fragile TikZ code—especially
for twin-axis (`twinx`) plots where the right y-axis margin is mis-calculated.
This library generates minimal, readable TikZ code that relies on a shared
LaTeX preamble to enforce consistent styling across an entire project.

---

## Plot types

| Builder | Use case |
|---------|----------|
| [`TwinPlot`] | Bar chart (energy efficiency, left y-axis) + line chart (throughput, right y-axis) |
| [`LinePlot`] | Single or multi-series line plots with IQR confidence bands |
| [`ZPlot`] | Efficiency vs. throughput scatter, one series per configuration group |

All plots compute **median** and **IQR** (Q1/Q3) across repeated measurements
automatically. Bar series use simple Q1-Q3 whiskers, while line series use a
transparent Q1-Q3 band.

---

## Quick start

```rust
use epy::prelude::*;

fn main() {
    let df = DataFrame::from_csv("results/gemm.csv").unwrap()
        .filter(|r| r["threads"] == 8.0)
        .with_column("gflop_j", |r| r["insns"] / r["rapl"] / 1e9)
        .with_column("gflop_s", |r| r["insns"] / r["runtime"] / 1e9);

    let grouped = df.group_by("powercap");

    // Twin-axis bar (efficiency) + line (throughput) plot
    let tikz = TwinPlot::new(grouped)
        .bar("gflop_j", r"\si{\giga\flop\per\joule}")
        .line("gflop_s", r"\si{\giga\flop\per\second}")
        .xlabel(r"Power limit (\si{\watt})")
        .render();

    std::fs::write("plots/gemm.tex", tikz).unwrap();
}
```

### Line plot with IQR band

```rust
let df = DataFrame::from_csv("results/gemm.csv").unwrap()
    .filter(|r| r["threads"] == 8.0)
    .with_column("ipc", |r| r["insns"] / r["cycs"]);

let tikz = LinePlot::new(df.group_by("powercap"))
    .series("ipc", palette::BLUE, "IPC")
    .xlabel(r"Power limit (\si{\watt})")
    .ylabel("IPC")
    .render();
```

### Z-plot (efficiency vs. throughput)

```rust
let df = DataFrame::from_csv("results/gemm.csv").unwrap()
    .with_column("gflop_j", |r| r["insns"] / r["rapl"] / 1e9)
    .with_column("gflop_s", |r| r["insns"] / r["runtime"] / 1e9);

let tikz = ZPlot::new(df.group_by("threads"))
    .x_col("gflop_s")
    .y_col("gflop_j")
    .xlabel(r"\si{\giga\flop\per\second}")
    .ylabel(r"\si{\giga\flop\per\joule}")
    .label_fn(|tc| format!("{} threads", tc as u32))
    .render();
```

---

## CSV format

The library expects a CSV file with numeric columns and a header row, for example:

```csv
size,threads,powercap,runtime,rapl,ina,insns,cycs,cycs-unc,temperature
100000000,1,25,0.288816,0.74536,1.75862,125469283,115376112,115425492,52
100000000,1,25,0.287199,0.736509,1.71418,125488489,114699426,114759614,51
```

Common derived columns:

| Column | Formula | Unit |
|--------|---------|------|
| `gflop_j` | `insns / rapl / 1e9` | GFLOP/J |
| `gflop_s` | `insns / runtime / 1e9` | GFLOP/s |
| `ipc` | `insns / cycs` | IPC |

---
