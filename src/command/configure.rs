use eyre::Result;
use thiserror::Error;

use crate::{provider::PROVIDER_TYPE_MAP, provider_loader::ProviderLoader};

/// Path to the config file
pub const CONFIG_PATH: &str = "config.json";

/// Errors for action configure
#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("Provider {provider} is not supported")]
    ProviderNotSupported { provider: String },
    #[error("Provider {provider} does not have API key")]
    NotHasAPIKey { provider: String },
}

/// Set default provider or API key for provider
pub fn set_provider<Loader: ProviderLoader>(
    provider: &str,
    api_key: Option<&str>,
    loader_args: Loader::ConstructorArg<'_>,
) -> Result<()> {
    // Check that provider is registered
    let provider_type =
        *PROVIDER_TYPE_MAP
            .get(provider)
            .ok_or(ConfigurationError::ProviderNotSupported {
                provider: provider.to_owned(),
            })?;

    let mut loader = Loader::new(loader_args)?;
    loader.load_config()?;

    // If an API key is provided, set it for the provider
    // otherwise provider has to be in config to become the default one
    if let Some(api_key) = api_key {
        loader.set_provider_key(provider_type, api_key)?
    } else if loader.get_config()?.keys.get(&provider_type).is_none() {
        return Err(ConfigurationError::NotHasAPIKey {
            provider: provider.to_owned(),
        }
        .into());
    };

    loader.set_default_provider(provider_type)?;
    loader.save_config()?;

    println!("Success! Provider {provider} is default provider now");

    Ok(())
}
