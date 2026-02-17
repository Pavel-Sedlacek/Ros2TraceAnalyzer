use std::path::PathBuf;
use clap::{Args, ValueHint};

#[derive(Debug, Clone, Args)]
pub struct ViewerArgs {
    /// The dotfile to open
    pub dotfile: PathBuf,

    #[clap(long, value_name = "xdot")]
    /// The entry point to the python viewer (defaults to system xdot binary)
    pub xdot: Option<String>,

    /// The executable to run to invoke the Ros2TraceAnalyzer (defaults to system Ros2TraceAnalyzer binary)
    #[clap(long, short = 't', value_name = "Ros2TraceAnalyzer")]
    pub tracer_exec: Option<String>,

    /// The directory with the datafiles (defaults to CWD)
    #[clap(long, short = 'd', value_name = "DATA", value_hint = ValueHint::DirPath)]
    pub data: Option<PathBuf>
}