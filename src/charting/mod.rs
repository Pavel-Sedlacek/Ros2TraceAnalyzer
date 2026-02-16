use std::path::PathBuf;

use plotters::{chart::{self, ChartBuilder, ChartContext}, coord::{CoordTranslate, ranged1d::ValueFormatter}, prelude::{BitMapBackend, Cartesian2d, DrawingBackend, IntoDrawingArea, Ranged}};
use plotters_svg::SVGBackend;

use crate::{argsv2::chart_args::{ChartRequest, ChartVariants}, charting::{axis_descriptor::{AxisBestFit, AxisDescriptors, resolve_axis_descriptors}, charts::{ChartData, histogram::HistogramChart, scatter::ScatterChart}, error::ChartConstructionError}, extract::ChartableData};

mod error;
mod charts;
mod axis_descriptor;

pub fn render_chart(
    file_name: &PathBuf,
    charting_data: &ChartableData,
    chart_request: &ChartRequest
) -> Result<(), ChartConstructionError> {
    fn label_axis<'a>(
        mut chart: ChartContext<'a, impl DrawingBackend, Cartesian2d<impl Ranged<ValueType = i64> + ValueFormatter<i64>, impl Ranged<ValueType = i64> + ValueFormatter<i64>>>,
        axis_best_fits: &[AxisBestFit; 2],
        axis_description: &AxisDescriptors
    ) -> Result<(), ChartConstructionError> {
        chart.configure_mesh()
            .x_desc(axis_description.x.label)
            .y_desc(axis_description.y.label)
            .x_label_formatter(&|v| format!("{:.2}", axis_best_fits[0].convert(*v) ).trim_end_matches('0').trim_end_matches('.').to_string() )
            .y_label_formatter(&|v| format!("{:.2}", axis_best_fits[1].convert(*v) ).trim_end_matches('0').trim_end_matches('.').to_string() )
            .draw()
            .map_err(|e| ChartConstructionError::InvalidCoordinateSystem(e.to_string()))?;

        Ok(())
    }

    fn draw_into_canvas(
        canvas: impl DrawingBackend,
        data: &ChartableData,
        variant: &ChartVariants,
        spacing: &ChartSpacing,
        axis_description: &AxisDescriptors,
    ) -> Result<(), ChartConstructionError> {
        let area = canvas.into_drawing_area();
        area.fill(&plotters::style::WHITE).unwrap();

        let mut chart = ChartBuilder::on(&area);

        chart
            .margin_left(spacing.margin[0])
            .margin_top(spacing.margin[1])
            .margin_right(spacing.margin[2])
            .margin_bottom(spacing.margin[3])
            .set_label_area_size(chart::LabelAreaPosition::Left, spacing.label[0])
            .set_label_area_size(chart::LabelAreaPosition::Top, spacing.label[1])
            .set_label_area_size(chart::LabelAreaPosition::Right, spacing.label[2])
            .set_label_area_size(chart::LabelAreaPosition::Bottom, spacing.label[3]);

        match &variant {
            ChartVariants::Histogram(histogram_data) => {
              let d = HistogramChart::new(histogram_data, data, axis_description);
                label_axis(
                    d.draw_into(&mut chart)?,
                    d.axis_fits(),
                    &axis_description
                )?;
            },
            ChartVariants::Scatter => {
                let d = ScatterChart::new(data, axis_description);
                label_axis(
                    d.draw_into(&mut chart)?,
                    d.axis_fits(),
                    &axis_description
                )?;
            }
        }

        area
            .present()
            .map_err(|e| ChartConstructionError::DrawingError(e.to_string()))?;
    
        Ok(())
    }

    let spacing = ChartSpacing {
        margin: [64, 64, 64, 64],
        label: [128, 0, 0, 64],
    };

    let axis_description = resolve_axis_descriptors(&chart_request.property, &chart_request.plot);

    match chart_request.output_format {
        crate::argsv2::chart_args::ChartOutputFormat::SVG => draw_into_canvas(
            SVGBackend::new(&file_name, (chart_request.size, chart_request.size)),
            charting_data,
            &chart_request.plot,
            &spacing,
            &axis_description
        ),
        crate::argsv2::chart_args::ChartOutputFormat::PNG => draw_into_canvas(
            BitMapBackend::new(&file_name, (chart_request.size, chart_request.size)),
            charting_data,
            &chart_request.plot,
            &spacing,
            &axis_description
        ),
    }?;

    Ok(())
}

struct ChartSpacing {
    pub margin: [i32; 4],
    pub label: [i32; 4]
}