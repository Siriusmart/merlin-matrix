use serde::{Deserialize, Serialize};

use crate::config::{ConfigDe, ConfigSerde};

/// Login credentials
#[derive(Serialize, Deserialize, Clone)]
pub struct CredsConfig {
    homeserver: String,
    username: String,
    password: String,
}

impl Default for CredsConfig {
    fn default() -> Self {
        Self {
            homeserver: "https://example.com".to_string(),
            username: "yourname".to_string(),
            password: "XXX".to_string(),
        }
    }
}

impl ConfigDe for CredsConfig {
    const PATH: &'static str = "creds";
}

impl ConfigSerde for CredsConfig {}

impl CredsConfig {
    pub fn homeserver(&self) -> &str {
        &self.homeserver
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}
