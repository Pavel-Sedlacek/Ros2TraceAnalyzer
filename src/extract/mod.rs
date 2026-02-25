use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use derive_more::Display;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::analyses::analysis::callback_duration::RecordExport;
use crate::analyses::analysis::dependency_graph::{
    ActivationDelayExport, MessagesDelayExport, PublicationDelayExport,
};
use crate::analyses::analysis::message_latency::MessageLatencyExport;
use crate::argsv2::extract_args::AnalysisProperty;
use crate::utils::binary_sql_store::{BinarySQLStore, BinarySQLStoreError};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Display)]
#[display("{node}::{interface}")]
pub struct RosInterfaceCompleteName {
    pub interface: String,
    pub node: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Display)]
#[display("{source_node}-({topic})>{target_node}")]
pub struct RosChannelCompleteName {
    pub source_node: String,
    pub target_node: String,
    pub topic: String,
}

pub enum ChartableData {
    I64(Vec<i64>),
}

#[derive(Error, Debug)]
pub enum DataExtractionError {
    #[error("There is no such analysis for element {0}")]
    NoSuchElement(String),
    #[error("An error occurred during data parsing\n{0}")]
    SourceDataParseError(BinarySQLStoreError),
}

pub fn extract(
    input: &Path,
    element_id: &str,
    property: &AnalysisProperty,
) -> color_eyre::eyre::Result<(String, ChartableData)> {
    let store = BinarySQLStore::new(input)?;

    if let AnalysisProperty::MessageLatencies = property {
        let id: RosChannelCompleteName = serde_qs::from_str(element_id)?;

        let f = store
            .read::<Vec<MessageLatencyExport>>(property.table_name())
            .map_err(DataExtractionError::SourceDataParseError)?;

        return f
            .into_iter()
            .find(|l| {
                l.source_node.eq(&id.source_node)
                    && l.target_node.eq(&id.target_node)
                    && l.topic.eq(&id.topic)
            })
            .map(|l| ChartableData::I64(l.latencies))
            .ok_or_else(|| DataExtractionError::NoSuchElement(id.to_string()))
            .map(|v| (property.table_name().to_owned(), v))
            .map_err(color_eyre::eyre::Report::new);
    }

    let id: RosInterfaceCompleteName = serde_qs::from_str(element_id)?;
    let v = match property {
        AnalysisProperty::CallbackDurations => store
            .read::<Vec<RecordExport>>(property.table_name())
            .map_err(DataExtractionError::SourceDataParseError)?
            .into_iter()
            .find(|r| r.caller.eq(&id.interface) && r.node.eq(&id.node))
            .map(|v| ChartableData::I64(v.durations)),
        AnalysisProperty::ActivationDelays => store
            .read::<Vec<ActivationDelayExport>>(property.table_name())
            .map_err(DataExtractionError::SourceDataParseError)?
            .into_iter()
            .find(|a| a.interface.eq(&id.interface) && a.node.eq(&id.node))
            .map(|v| ChartableData::I64(v.activation_delays)),
        AnalysisProperty::PublicationDelays => store
            .read::<Vec<PublicationDelayExport>>(property.table_name())
            .map_err(DataExtractionError::SourceDataParseError)?
            .into_iter()
            .find(|a| a.interface.eq(&id.interface) && a.node.eq(&id.node))
            .map(|v| ChartableData::I64(v.publication_delays)),
        AnalysisProperty::MessageDelays => store
            .read::<Vec<MessagesDelayExport>>(property.table_name())
            .map_err(DataExtractionError::SourceDataParseError)?
            .into_iter()
            .find(|a| a.interface.eq(&id.interface) && a.node.eq(&id.node))
            .map(|v| ChartableData::I64(v.messages_delays)),
        _ => {
            unreachable!()
        }
    }
    .ok_or_else(|| DataExtractionError::NoSuchElement(id.to_string()))
    .map_err(|e| color_eyre::eyre::Report::new(e))?;

    Ok((property.table_name().to_owned(), v))
}

impl ChartableData {
    pub fn export(&self, output: &Path) -> color_eyre::eyre::Result<()> {
        let mut f = File::create(output)?;

        let data = match self {
            ChartableData::I64(items) => serde_json::to_string(&items)?,
        };

        f.write_all(data.as_bytes())?;

        Ok(())
    }
}
