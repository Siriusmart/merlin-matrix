use std::{env, error::Error, fs, path::PathBuf};

use serde::{Serialize, de::DeserializeOwned};

pub mod creds;
pub mod data;
pub mod handlers;
pub mod register;

pub use register::*;

pub trait ConfigDe: DeserializeOwned + Default {
    const PATH: &'static str;

    fn path() -> PathBuf {
        PathBuf::from(env::var("CONFIG_ROOT").expect("missing env CONFIG_ROOT"))
            .join(Self::PATH)
            .with_added_extension("toml")
    }

    fn load() -> Result<Self, Box<dyn Error>> {
        let path = Self::path();

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read(path)?;
        Ok(toml::from_slice(&content)?)
    }
}

pub trait ConfigSerde: Serialize + ConfigDe {
    fn write(&self) -> Result<(), Box<dyn Error>> {
        let path = Self::path();

        if !path.parent().unwrap().exists() {
            fs::create_dir_all(path.parent().unwrap())?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;

        Ok(())
    }

    fn load_write() -> Result<Self, Box<dyn Error>> {
        let path = Self::path();

        if path.exists() {
            Self::load()
        } else {
            let default = Self::default();
            default.write()?;
            Ok(default)
        }
    }
}
