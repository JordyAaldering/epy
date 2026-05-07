//! Plot builders for energy-measurement data.
mod line;
mod twin;
mod zplot;

pub use line::LinePlot;
pub use twin::TwinPlot;
pub use zplot::ZPlot;

use crate::{color::Color, data::GroupedFrame, ir::*, stats::*};

pub(crate) fn common_axis_options() -> Vec<AxisOption> {
    vec![
        AxisOption::flag("scale only axis"),
        AxisOption::key_value("axis line style", format!("{{{}}}", Color::Grid.tikz_name())),
        AxisOption::key_value("y grid style", format!("{{{}}}", Color::Grid.tikz_name())),
        AxisOption::flag("ymajorgrids"),
        AxisOption::key_value("major tick length", "3pt"),
        AxisOption::key_value("xtick style", format!("{{color={}}}", Color::Grid.tikz_name())),
        AxisOption::key_value("ytick style", format!("{{color={}}}", Color::Grid.tikz_name())),
        AxisOption::key_value("tick label style", r"{font=\epyticksize, inner sep=2pt}"),
        AxisOption::key_value("legend style", format!("{{font=\\epylegendsize,draw={},fill opacity=0.8,draw opacity=1,text opacity=1}}", Color::Grid.tikz_name())),
        // Phantom tick at ymax ensures consistent axis height across plots
        AxisOption::key_value("extra y ticks", r"{\pgfkeysvalueof{/pgfplots/ymax}}"),
        AxisOption::key_value("extra y tick labels", r"{\vphantom{Ag}}"),
        AxisOption::key_value("extra y tick style", "{yticklabel style={opacity=0,text opacity=0},major tick length=0pt}"),
    ]
}

/// Compute per-group statistics for `col` from a [`GroupedFrame`].
pub(crate) fn group_stats(gf: &GroupedFrame, col: &str) -> Vec<IQR> {
    (0..gf.num_groups())
        .map(|gi| {
            let xs = gf.group_values(gi, col);
            IQR {
                median: median(&xs),
                q1: q1(&xs),
                q3: q3(&xs),
            }
        })
        .collect()
}
