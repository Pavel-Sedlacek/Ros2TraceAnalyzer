use std::{fs::File, io::Write, path::PathBuf};

use derive_more::Display;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{analyses::analysis::{callback_duration::RecordExport, dependency_graph::{ActivationDelayExport, MessagesDelayExport, MessagesLatenciesExport, NodeType, PublicationDelayExport}}, argsv2::extract_args::AnalysisProperty, utils::binary_sql_store::{BinarySQLStore, BinarySQLStoreError}};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Display)]
#[display("{namespace}::{interface}")]
pub struct RosInterfaceCompleteName {
    pub interface: String,
    pub namespace: String,
    pub interface_type: NodeType
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Display)]
#[display("{source_namespace}-({identifier})>{target_namespace}")]
pub struct RosChannelCompleteName {
    pub source_namespace: String,
    pub target_namespace: String,
    pub identifier: String,
}

#[derive(Clone, Debug)]
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

    match property {
        AnalysisProperty::MessagesLatency => {
            let id: RosChannelCompleteName = serde_qs::from_str(&element_id)?;

            let f = store
                .read::<Vec<MessagesLatenciesExport>>("message_latencies")
                .map_err(|e| DataExtractionError::SourceDataParseError(e))?;

            return f.into_iter()
                .find(|l| l.from_namespace.eq(&id.source_namespace) && l.to_namespace.eq(&id.target_namespace) && l.identifier.eq(&id.identifier))
                .map(|l| ChartableData::I64(l.latencies))
                .ok_or_else(|| DataExtractionError::NoSuchElement(id.to_string()))
                .map(|v| ("message_latencies".to_string(), v))
                .map_err(|e| color_eyre::eyre::Report::new(e))
        }
        _ => {}
    }

    let id: RosInterfaceCompleteName = serde_qs::from_str(&element_id)?;
    match property {
        AnalysisProperty::CallbackDuration => {
            let f = store
                .read::<Vec<RecordExport>>("callback_duration")
                .map_err(|e| DataExtractionError::SourceDataParseError(e))?;

            f.into_iter()
                .find(|r| r.caller.eq(&id.interface) && r.node.eq(&id.namespace))
                .map(|v| ChartableData::I64(Vec::from(v.durations)))
                .ok_or_else(|| DataExtractionError::NoSuchElement(id.to_string()))
                .map(|v| ("callback_duration".to_string(), v))
        }
        AnalysisProperty::ActivationsDelay => {
            let f = store
                .read::<Vec<ActivationDelayExport>>("activation_delays")
                .map_err(|e| DataExtractionError::SourceDataParseError(e))?;

            f.into_iter()
                .find(|a| a.interface.eq(&id.interface) && a.node.eq(&id.namespace))
                .map(|v| ChartableData::I64(v.activation_delays))
                .ok_or_else(|| DataExtractionError::NoSuchElement(id.to_string()))
                .map(|v| ("activation_delays".to_string(), v))
        }
        AnalysisProperty::PublicationsDelay => {
            let f = store
                .read::<Vec<PublicationDelayExport>>("publication_delays")
                .map_err(|e| DataExtractionError::SourceDataParseError(e))?;

            f.into_iter()
                .find(|a| a.interface.eq(&id.interface) && a.node.eq(&id.namespace))
                .map(|v| ChartableData::I64(v.publication_delays))
                .ok_or_else(|| DataExtractionError::NoSuchElement(id.to_string()))
                .map(|v| ("publication_delays".to_string(), v))
        }
        AnalysisProperty::MessagesDelay => {
            let f = store
                .read::<Vec<MessagesDelayExport>>("message_delays")
                .map_err(|e| DataExtractionError::SourceDataParseError(e))?;

            f.into_iter()
                .find(|a| a.interface.eq(&id.interface) && a.node.eq(&id.namespace))
                .map(|v| ChartableData::I64(v.messages_delays))
                .ok_or_else(|| DataExtractionError::NoSuchElement(id.to_string()))
                .map(|v| ("message_delays".to_string(), v))
        }
        _ => {
            unreachable!()
        }
    }.map_err(|e| color_eyre::eyre::Report::new(e))
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
