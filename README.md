# energy-plots

A small Rust DSL for generating clean [PGFPlots](https://ctan.org/pkg/pgfplots) /
TikZ code from CSV energy-measurement data.

---

## Why?

The Python `matplot2tikz` library produces verbose, fragile TikZ code—especially
for twin-axis (`twinx`) plots where the right y-axis margin is mis-calculated.
This library generates minimal, readable TikZ code that relies on a shared
`pgfplotsset` prelude to enforce consistent styling across an entire project.

---

## Plot types

| Builder | Use case |
|---------|----------|
| [`BarLinePlot`] | Bar chart (energy efficiency, left y-axis) + line chart (throughput, right y-axis) |
| [`LinePlot`] | Single or multi-series line plots with IQR confidence bands |
| [`ZPlot`] | Efficiency vs. throughput scatter, one series per configuration group |

All plots compute **median** and **IQR** (Q1/Q3) across repeated measurements
automatically.

---

## Quick start

```rust
use energy_plots::prelude::*;

fn main() {
    let df = DataFrame::from_csv("results/gemm.csv").unwrap()
        .filter(|r| r["threads"] == 8.0)
        .with_column("gflop_j", |r| r["insns"] / r["rapl"] / 1e9)
        .with_column("gflop_s", |r| r["insns"] / r["runtime"] / 1e9);

    let grouped = df.group_by("powercap");

    // Twin-axis bar (efficiency) + line (throughput) plot
    let tikz = BarLinePlot::new(grouped)
        .bar("gflop_j", palette::GREEN, r"\si{\giga\flop\per\joule}")
        .line("gflop_s", palette::RED,  r"\si{\giga\flop\per\second}")
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

## LaTeX prelude

The generated TikZ code references `common/*` pgfplots styles.
Add the following to your LaTeX document preamble (e.g. in `preamble.tex`):

```latex
\usepackage{pgfplots}
\usepgfplotslibrary{groupplots,dateplot}
\usetikzlibrary{patterns,shapes.arrows}

\newlength{\yaxispadding}
\setlength{\yaxispadding}{3.5em}

\newif\ifcommonplotxgrid
\commonplotxgridfalse

\definecolor{commonplottext}{RGB}{38,38,38}
\definecolor{commonplotgrid}{RGB}{204,204,204}
\definecolor{commonplottick}{RGB}{192,192,192}

\pgfplotsset{
  compat=newest,
  common/twin-shared/.style={
    scale only axis,
    width={\dimexpr \linewidth - \yaxispadding\relax},
    axis line style={commonplotgrid},
    x grid style={commonplotgrid},
    y grid style={commonplotgrid},
    label style={font=\scriptsize,text=commonplottext,inner sep=0pt},
    tick label style={font=\scriptsize,text=commonplottext,inner sep=2pt},
    legend style={font=\scriptsize,draw=commonplotgrid,fill opacity=0.8,draw opacity=1,text opacity=1},
    major tick length=3pt,
    major tick style={draw=commonplotgrid},
    xtick style={color=commonplottick},
    ytick style={color=commonplottick},
  },
  common/line/.style={
    mark size=2pt,
  },
  common/bar/.style={
  },
  common/twin-main/.style={
    common/twin-shared,
    name=mainaxis,
    trim axis right,
  },
  common/twin/.style={
    common/twin-shared,
    at={(mainaxis.south west)},
    anchor=south west,
    trim axis left,
    axis x line=none,
    xmajorgrids=false,
    ymajorgrids=false,
    xtick=\empty,
    xticklabels=\empty,
    ylabel style={font=\scriptsize},
    yticklabel style={font=\scriptsize},
  },
  every axis/.append style={
    axis on top=false,
    axis line style={commonplotgrid},
    x grid style={commonplotgrid},
    y grid style={commonplotgrid},
    xmajorgrids=\ifcommonplotxgrid true\else false\fi,
    ymajorgrids,
    label style={font=\scriptsize,text=commonplottext,inner sep=2pt},
    tick label style={font=\scriptsize,text=commonplottext,inner sep=2pt},
    legend style={font=\scriptsize,draw=commonplotgrid,fill opacity=0.8,draw opacity=1,text opacity=1},
    xtick style={color=commonplottick},
    ytick style={color=commonplottick},
    major tick length=3pt,
  },
  every axis x label/.append style={font=\scriptsize,text=commonplottext},
  every axis y label/.append style={font=\scriptsize,text=commonplottext},
  every axis x tick label/.append style={font=\scriptsize,text=commonplottext},
  every axis y tick label/.append style={font=\scriptsize,text=commonplottext},
  every axis/.append style={
    extra y ticks={\pgfkeysvalueof{/pgfplots/ymax}},
    extra y tick labels={\vphantom{Ag}},
    extra y tick style={yticklabel style={opacity=0,text opacity=0},major tick length=0pt},
  },
  legend image code/.code={\draw[#1] (0pt,0pt) -- (1em,0pt);\path[#1,mark size=1.75*\pgfplotmarksize] plot coordinates {(0.5em,0pt)};},
  ybar legend/.style={legend image code/.code={\draw[#1,draw=none] (0pt,-0.22em) rectangle (1em,0.22em);}},
}
```

---

## Building

```sh
cargo build
cargo test
```
