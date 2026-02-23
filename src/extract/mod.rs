use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

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
#[display("{namespace}::{interface}")]
pub struct RosInterfaceCompleteName {
    pub interface: String,
    pub namespace: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Display)]
#[display("{source_namespace}-({topic})>{target_namespace}")]
pub struct RosChannelCompleteName {
    pub source_namespace: String,
    pub target_namespace: String,
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
    input: PathBuf,
    element_id: &str,
    property: &AnalysisProperty,
) -> color_eyre::eyre::Result<(String, ChartableData)> {
    let store = BinarySQLStore::new(input)?;

    if let AnalysisProperty::MessagesLatency = property {
        let id: RosChannelCompleteName = serde_qs::from_str(element_id)?;

        let f = store
            .read::<Vec<MessageLatencyExport>>(property.table_name())
            .map_err(DataExtractionError::SourceDataParseError)?;

        return f
            .into_iter()
            .find(|l| {
                l.source_node.eq(&id.source_namespace)
                    && l.target_node.eq(&id.target_namespace)
                    && l.topic.eq(&id.topic)
            })
            .map(|l| ChartableData::I64(l.latencies))
            .ok_or_else(|| DataExtractionError::NoSuchElement(id.to_string()))
            .map(|v| (property.table_name().to_owned(), v))
            .map_err(color_eyre::eyre::Report::new);
    }

    let id: RosInterfaceCompleteName = serde_qs::from_str(element_id)?;
    let v = match property {
        AnalysisProperty::CallbackDuration => store
            .read::<Vec<RecordExport>>(property.table_name())
            .map_err(DataExtractionError::SourceDataParseError)?
            .into_iter()
            .find(|r| r.caller.eq(&id.interface) && r.node.eq(&id.namespace))
            .map(|v| ChartableData::I64(v.durations)),
        AnalysisProperty::ActivationsDelay => store
            .read::<Vec<ActivationDelayExport>>(property.table_name())
            .map_err(DataExtractionError::SourceDataParseError)?
            .into_iter()
            .find(|a| a.interface.eq(&id.interface) && a.node.eq(&id.namespace))
            .map(|v| ChartableData::I64(v.activation_delays)),
        AnalysisProperty::PublicationsDelay => store
            .read::<Vec<PublicationDelayExport>>(property.table_name())
            .map_err(DataExtractionError::SourceDataParseError)?
            .into_iter()
            .find(|a| a.interface.eq(&id.interface) && a.node.eq(&id.namespace))
            .map(|v| ChartableData::I64(v.publication_delays)),
        AnalysisProperty::MessagesDelay => store
            .read::<Vec<MessagesDelayExport>>(property.table_name())
            .map_err(DataExtractionError::SourceDataParseError)?
            .into_iter()
            .find(|a| a.interface.eq(&id.interface) && a.node.eq(&id.namespace))
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
    pub fn export(&self, output: PathBuf) -> color_eyre::eyre::Result<()> {
        let mut f = File::create(output)?;

        let data = match self {
            ChartableData::I64(items) => serde_json::to_string(&items)?,
        };

        f.write_all(data.as_bytes())?;

        Ok(())
    }
}
