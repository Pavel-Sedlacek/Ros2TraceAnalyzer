use std::path::PathBuf;
use clap::{Args, Subcommand, ValueEnum, ValueHint};
use derive_more::Display;

use crate::argsv2::extract_args::AnalysisProperty;

const DEFAULT_BUNDLE_NAME: &'static str = "binary_bundle.sqlite";

#[derive(Debug, Clone, Args)]
pub struct ChartArgs {
    /// Identifier of the element for which to draw the graph
    /// 
    /// - For nodes (graphviz nodes) the namespace (ROS node), type (ROS interface) and parameters (ROS topic) need to be specified  
    /// - For edges (graphviz edges) name (type + topic) of the source and target node should be provided
    /// 
    /// The expected format is URL encoded map
    #[clap(long, short = 'n')]
    element_id: String,

    /// The input path, either a file of the data or a folder containing the default named file with the necessary data
    #[clap(long, short = 'i', value_name = "INPUT", value_hint = ValueHint::AnyPath)]
    input_path: Option<PathBuf>,

    /// The output path, either a folder to which the file will be generated or a file to write into
    #[clap(long, short = 'o', value_name = "OUTPUT", value_hint = ValueHint::AnyPath)]
    output_path: Option<PathBuf>,

    /// Whether the chart should be rerender from scratch
    ///
    /// if not set a preexisting chart will be used only if it matches all parameters
    #[clap(long, short = 'c', default_value = "false")]
    clean: bool,

    #[clap(flatten)]
    chart: ChartRequest,
}

impl ChartArgs {
    pub fn element_id(&self) -> &str {
        &self.element_id
    }


    pub fn input_path(&self) -> PathBuf {
        match &self.input_path {
            Some(p) => {
                if p.is_dir() {
                    p.join(DEFAULT_BUNDLE_NAME)
                } else {
                    p.clone()
                }
            },
            None => std::env::current_dir().unwrap().join(DEFAULT_BUNDLE_NAME)
        }
    }

    pub fn output_path(&self) -> &Option<PathBuf> {
        &self.output_path
    }

    pub fn clean(&self) -> bool {
        self.clean
    }

    pub fn chart(&self) -> &ChartRequest {
        &self.chart
    }
}

#[derive(Debug, Display, Args, Clone)]
#[display("ChartOf {{ value: {property}, {plot} }}")]
pub struct ChartRequest {
    /// The value to plot into the chart
    #[clap(long)]
    pub property: AnalysisProperty,

    /// The type of chart to render the data as
    #[command(subcommand)]
    pub plot: ChartVariants,

    /// The size of the rendered image
    #[clap(long, default_value = "800")]
    pub size: u32,

    /// The filetype (output format) the rendered image should be in
    #[clap(long, default_value_t = ChartOutputFormat::default())]
    pub output_format: ChartOutputFormat
}

impl ChartRequest {
    pub(crate) fn name_descriptor(&self) -> String {
        let value = match self.property {
            AnalysisProperty::CallbackDuration => "execution_timing",
            AnalysisProperty::ActivationsDelay => "activations_delay",
            AnalysisProperty::PublicationsDelay => "publication_delay",
            AnalysisProperty::MessagesDelay => "message_delay",
            AnalysisProperty::MessagesLatency => "latency",
        };

        let plot = match &self.plot {
            ChartVariants::Histogram(hist_data) => format!("histogram_{}", hist_data.bins.unwrap_or(0)),
            ChartVariants::Scatter => "scatter".to_owned(),
        };

        format!("{}_{}_{}", value, plot, self.size)
    }
}

#[derive(Debug, Display, ValueEnum, Clone, Default)]
pub enum ChartOutputFormat {
    #[default]
    #[display("svg")]
    SVG,
    #[display("png")]
    PNG
}


#[derive(Debug, Display, Subcommand, Clone)]
pub enum ChartVariants {
    #[display("Histogram")]
    Histogram(HistogramData),
    #[display("Scatter")]
    Scatter,
}

#[derive(Debug, Display, Args, Clone)]
#[display("Histogram data {{ bins: {bins:?} }}")]
pub struct HistogramData {
    /// Number of bins to split the data into
    #[arg(long, short = 'b', value_name = "BINS")]
    pub bins: Option<usize>
}