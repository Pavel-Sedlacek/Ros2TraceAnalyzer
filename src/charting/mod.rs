use std::path::PathBuf;

use plotters::{chart::{self, ChartBuilder, ChartContext}, coord::{CoordTranslate, ranged1d::ValueFormatter}, prelude::{BitMapBackend, Cartesian2d, DrawingBackend, IntoDrawingArea, Ranged}};
use plotters_svg::SVGBackend;

use crate::{argsv2::chart_args::{ChartRequest, ChartVariants}, charting::{axis_descriptor::{AxisBestFit, AxisDescriptor, AxisDescriptors, resolve_axis_descriptors}, charts::{ChartData, histogram::HistogramChart, scatter::ScatterChart}, error::ChartConstructionError}, extract::ChartableData};

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
        fn format_axis(axis_description: &AxisDescriptor, best_fit: &AxisBestFit) -> String {
            let unit_name = format!("{}{}", best_fit.target_notion(), axis_description.unit_name());

            if !unit_name.is_empty() {
                format!("{} [{}]", axis_description.label, unit_name)
            } else {
                axis_description.label.to_string()
            }
        }

        chart.configure_mesh()
            .x_desc(format_axis(&axis_description.x, &axis_best_fits[0]))
            .y_desc(format_axis(&axis_description.y, &axis_best_fits[1]))
            .x_label_formatter(&|v| format!("{:.2}", axis_best_fits[0].convert(*v) ).trim_end_matches('0').trim_end_matches('.').to_string() )
            .y_label_formatter(&|v| format!("{:.2}", axis_best_fits[1].convert(*v) ).trim_end_matches('0').trim_end_matches('.').to_string() )
            .x_labels(9)
            .max_light_lines(1)
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
    
        drop(area);
        
        Ok(())
    }

    let spacing = ChartSpacing {
        margin: [(chart_request.size / 10) as i32; 4],
        label: [(chart_request.size / 10) as i32, 0, 0, (chart_request.size / 10) as i32],
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