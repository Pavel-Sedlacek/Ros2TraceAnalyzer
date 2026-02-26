use std::fs::File;
use std::io::Write;
use std::path::Path;

use derive_more::Display;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::argsv2::extract_args::AnalysisProperty;
use crate::utils::binary_sql_store::{BinarySQLStore, BinarySQLStoreError};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Display, Debug)]
#[display("{node}::{interface}")]
pub struct RosInterfaceCompleteName {
    pub interface: String,
    pub node: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Display, Debug)]
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
    #[error("An error occurred during data parsing\n{0}")]
    SourceDataParseError(BinarySQLStoreError),
}

pub fn extract(
    input: &Path,
    element_id: &str,
    property: &AnalysisProperty,
) -> color_eyre::eyre::Result<(String, ChartableData)> {
    let store = BinarySQLStore::new(input)?;

    let id = match property {
        AnalysisProperty::MessageLatencies => {
            serde_qs::from_str::<RosChannelCompleteName>(element_id)?.to_string()
        }
        _ => serde_qs::from_str::<RosInterfaceCompleteName>(element_id)?.to_string(),
    };

    Ok((
        property.table_name().to_owned(),
        ChartableData::I64(
            store
                .read::<Vec<i64>>(property.table_name(), &id)
                .map_err(DataExtractionError::SourceDataParseError)?,
        ),
    ))
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
