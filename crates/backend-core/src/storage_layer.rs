use std::{path::PathBuf, collections::HashMap};

use multiversx_sc::types::Address;
use serde_with::serde_as;
use crate::handlers::TokenDefinition;

pub type EventHash = Vec<u8>;

#[serde_as]
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageData {
    last_fetched_block: u64,
    already_parsed_events_on_last_fetched_block: Vec<EventHash>,
    #[serde_as(as = "Vec<(_, _)>")]
    pub balances: HashMap<String, Vec<TokenDefinition>>,
}

impl StorageData {
    pub fn new() -> Self {
        Self {
            last_fetched_block: 0,
            already_parsed_events_on_last_fetched_block: vec![],
            balances: HashMap::new(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn add_event_hash(&mut self, event_hash: EventHash) {
        self.already_parsed_events_on_last_fetched_block
            .push(event_hash);
    }

    pub fn set_last_fetched_block(&mut self, block: u64) {
        self.last_fetched_block = block;
        self.already_parsed_events_on_last_fetched_block.clear();
    }

    #[tracing::instrument(ret, err, skip(storage_fs_path))]
    pub async fn write_to_disk(&self, storage_fs_path: &PathBuf) -> eyre::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        tokio::fs::write(storage_fs_path, json).await?;
        Ok(())
    }

    #[tracing::instrument(ret, skip(storage_fs_path))]
    pub fn read_from_disk(storage_fs_path: &PathBuf) -> Self {
        let data = fast_read_and_parse(storage_fs_path).unwrap_or_else(|_| Self::new());
        let _ = data.write_to_disk(storage_fs_path);
        data
    }

    pub fn last_fetched_block(&self) -> u64 {
        self.last_fetched_block
    }

    pub fn contains(&self, x: &EventHash) -> bool {
        self.already_parsed_events_on_last_fetched_block.contains(x)
    }
}

fn fast_read_and_parse(storage_fs_path: &PathBuf) -> Result<StorageData, eyre::Error> {
    let json = std::fs::read_to_string(storage_fs_path)?;
    let data = serde_json::from_str(&json)?;
    Ok(data)
}
