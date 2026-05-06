mod color;
mod data;
mod ir;
mod plot;
mod stats;

pub mod prelude {
    pub use crate::color::Color;
    pub use crate::data::{DataFrame, GroupedFrame};
    pub use crate::plot::{TwinPlot, LinePlot, ZPlot};
    pub use crate::stats::{mean, median, q1, q3};
}
