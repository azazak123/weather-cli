use eyre::Result;
use std::{
    fs::{read_to_string, write},
    io,
};
use thiserror::Error;

use super::ProviderLoader;
use crate::{config::Config, provider::ProviderType};

/// Errors for JsonLoader
#[derive(Error, Debug)]
pub enum JsonLoaderError {
    #[error("Failed to read config from {path} with error '{error}'")]
    FailedToRead { error: io::Error, path: String },
    #[error("Failed to parse config {path} with error '{error}'")]
    FailedToParse {
        error: serde_json::Error,
        path: String,
    },
    #[error("Config has not been loaded")]
    ConfigNotLoaded,
    #[error("Failed to stringify config with error '{error}'")]
    FailedToStringify { error: serde_json::Error },
    #[error("Failed to save config in {path} error {error}")]
    FailedToSave { error: io::Error, path: String },
    #[error("Failed to set default provider {provider:?}")]
    FailedToSetDefaultProvider { provider: ProviderType },
    #[error("Failed to set API key for provider {provider:?}")]
    FailedToSetAPI { provider: ProviderType },
}

/// Loader for saving config and loading providers from JSON
#[derive(Default)]
pub struct JsonLoader {
    path: String,
    config: Option<Config>,
}

impl ProviderLoader for JsonLoader {
    type ConstructorArg<'a> = &'a str;

    fn new(path: Self::ConstructorArg<'_>) -> Result<Self> {
        let mut loader = {
            JsonLoader {
                path: path.to_owned(),
                config: None,
            }
        };

        // In case of lack of config JSON file, we should create it
        if let Err(error) = loader.load_config() {
            if let Some(JsonLoaderError::FailedToRead { .. }) =
                error.downcast_ref::<JsonLoaderError>()
            {
                loader.create_config_file()?;
                loader.load_config()?;
            };
        }

        Ok(loader)
    }

    fn load_config(&mut self) -> Result<()> {
        let raw_config =
            read_to_string(&self.path).map_err(|error| JsonLoaderError::FailedToRead {
                error,
                path: self.path.to_owned(),
            })?;

        let config = serde_json::from_str::<Config>(&raw_config).map_err(|error| {
            JsonLoaderError::FailedToParse {
                error,
                path: self.path.to_owned(),
            }
        })?;

        self.config = Some(config);

        Ok(())
    }

    fn get_config(&self) -> Result<&Config> {
        let config = self
            .config
            .as_ref()
            .ok_or(JsonLoaderError::ConfigNotLoaded)?;

        Ok(config)
    }

    fn save_config(&self) -> Result<()> {
        let raw_config = serde_json::to_string(&self.get_config()?)
            .map_err(|error| JsonLoaderError::FailedToStringify { error })?;

        write(&self.path, raw_config).map_err(|error| JsonLoaderError::FailedToSave {
            error,
            path: self.path.to_owned(),
        })?;

        Ok(())
    }

    fn set_default_provider(&mut self, provider: ProviderType) -> Result<()> {
        let mut config = self
            .config
            .take()
            .ok_or(JsonLoaderError::FailedToSetDefaultProvider { provider })?;

        config.default = Some(provider);

        self.config = Some(config);

        Ok(())
    }

    fn set_provider_key(&mut self, provider: ProviderType, key: &str) -> Result<()> {
        let mut config = self
            .config
            .take()
            .ok_or(JsonLoaderError::FailedToSetAPI { provider })?;

        config.keys.insert(provider, key.to_owned());

        self.config = Some(config);

        Ok(())
    }
}

impl JsonLoader {
    /// Create and save blank config
    fn create_config_file(&self) -> Result<()> {
        let loader: Self = Default::default();

        loader.save_config()?;

        Ok(())
    }
}
