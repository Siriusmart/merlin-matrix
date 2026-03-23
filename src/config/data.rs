use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::{ConfigDe, ConfigSerde};

#[derive(Serialize, Deserialize, Clone)]
pub struct DataConfig {
    sqlite_db_path: PathBuf,
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            sqlite_db_path: PathBuf::from("./database.sqlite"),
        }
    }
}

impl ConfigDe for DataConfig {
    const PATH: &'static str = "data";
}

impl ConfigSerde for DataConfig {}

impl DataConfig {
    pub fn sqlite_db_path(&self) -> &Path {
        &self.sqlite_db_path
    }
}
