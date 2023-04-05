use eyre::Result;
use thiserror::Error;

use crate::{
    config::Config,
    provider::{Provider, ProviderIR, ProviderType},
};

pub mod json_loader;

/// Errors for ProviderLoader
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("API key for provider {provider:?} is not found")]
    APIKeyNotFound { provider: ProviderType },
    #[error("Default provider has not set")]
    DefaultProviderNotSet,
}

/// Trait for provider loader
/// Can be used for creating different ways for load provider (from JSON, YAML, database, etc.)
pub trait ProviderLoader {
    /// Arguments of the loader constructor
    type ConstructorArg<'a>;

    fn new(arg: Self::ConstructorArg<'_>) -> Result<Self>
    where
        Self: Sized;

    /// Load config from some place
    fn load_config(&mut self) -> Result<()>;

    /// Save config to some place
    fn save_config(&self) -> Result<()>;

    /// Get cached config
    fn get_config(&self) -> Result<&Config>;

    /// Set default weather provider
    fn set_default_provider(&mut self, provider: ProviderType) -> Result<()>;

    /// Set API key for specified weather provider
    fn set_provider_key(&mut self, provider: ProviderType, key: &str) -> Result<()>;

    /// Get provider by provider type
    fn get_provider(&self, provider: ProviderType) -> Result<Provider> {
        let key = self
            .get_config()?
            .keys
            .get(&provider)
            .ok_or(LoaderError::APIKeyNotFound { provider })?
            .clone();

        Ok(ProviderIR { key, provider }.into())
    }

    /// Get default provider
    fn get_default_provider(&self) -> Result<Provider> {
        let config = self.get_config()?;
        let default_provider = config.default.ok_or(LoaderError::DefaultProviderNotSet)?;

        self.get_provider(default_provider)
    }
}
