use plotters::{chart::{ChartBuilder, ChartContext}, coord::CoordTranslate, prelude::DrawingBackend};

use crate::charting::{axis_descriptor::{AxisBestFit}, error::ChartConstructionError};

pub mod histogram;
pub mod scatter;

pub trait ChartData<C: CoordTranslate> {
    fn axis_fits(&self) -> &[AxisBestFit; 2]; 
    fn draw_into<'a, B: DrawingBackend>(&self, canvas: &mut ChartBuilder<B>) -> Result<ChartContext<'a, B, C>, ChartConstructionError> ;
}

pub fn resolve_axis_range(
    data: &[i64],
) -> (i64, i64) {
    return (*data.iter().min().unwrap_or(&0), *data.iter().max().unwrap_or(&0))
}