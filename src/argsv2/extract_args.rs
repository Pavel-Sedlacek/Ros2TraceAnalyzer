use std::path::{Path, PathBuf};

use clap::{Args, ValueEnum, ValueHint};
use derive_more::Display;

use crate::argsv2::analysis_args::filenames;

#[derive(Debug, Clone, Args)]
pub struct ExtractArgs {
    /// Identifies the element in the dependency graph for
    /// which to extract the data
    ///
    /// - Nodes (graph nodes) are identified by the ROS node,
    ///   type (ROS interface) and parameters (ROS topic)
    ///
    /// - For edges (graphviz edges) name (type + topic)
    ///   of the source and target node should be provided
    ///
    /// The expected format is URL-encoded set of the required properties
    ///
    /// Example node id:
    /// `interface=Callback(Subscriber(%22/clock%22)&node=/abc`
    ///
    /// Example edge id:
    /// `source_node=/abc&target_node=/def&identifier=/some/topic`
    element_id: String,

    /// The property to extract from the node.
    property: AnalysisProperty,

    /// The input path, either a file of the data or a folder containing the default named file with the necessary data
    #[clap(long, short = 'i', value_name = "INPUT", value_hint = ValueHint::AnyPath)]
    input_path: Option<PathBuf>,

    /// The output path, either a folder to which the file will be generated or a file to write into
    #[clap(long, short = 'o', value_name = "OUTPUT", value_hint = ValueHint::FilePath)]
    output_path: PathBuf,
}

impl ExtractArgs {
    pub fn element_id(&self) -> &str {
        &self.element_id
    }

    pub fn property(&self) -> &AnalysisProperty {
        &self.property
    }

    pub fn input_path(&self) -> PathBuf {
        match &self.input_path {
            Some(p) => {
                if p.is_dir() {
                    p.join(filenames::BINARY_BUNDLE)
                } else {
                    p.clone()
                }
            }
            None => std::env::current_dir()
                .unwrap()
                .join(filenames::BINARY_BUNDLE),
        }
    }

    pub fn output_path(&self) -> &Path {
        self.output_path.as_path()
    }
}

#[derive(Debug, Display, ValueEnum, Clone)]
pub enum AnalysisProperty {
    /// Callback execution durations
    #[display("Callback execution time")]
    CallbackDurations,

    /// Delays between callback or timer activations
    #[display("Delays between activations")]
    ActivationDelays,

    /// Delays between publisher publications
    #[display("Delay between publication")]
    PublicationDelays,

    /// Delays between subscriber messages
    #[display("Delay between")]
    MessageDelays,

    /// Latency of a communication channel
    #[display("Message latency")]
    MessageLatencies,
}

impl AnalysisProperty {
    /// Table name in the binary data blob for this property
    pub fn table_name(&self) -> &'static str {
        match self {
            AnalysisProperty::CallbackDurations => "callback_duration",
            AnalysisProperty::ActivationDelays => "activations_delay",
            AnalysisProperty::PublicationDelays => "publications_delay",
            AnalysisProperty::MessageDelays => "messages_delay",
            AnalysisProperty::MessageLatencies => "messages_latency",
        }
    }
}
