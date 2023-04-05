use clap::{Parser, Subcommand};
use eyre::Result;

use self::configure::{set_provider, CONFIG_PATH};
use self::get::get;
use crate::provider_loader::json_loader::JsonLoader;

mod configure;
mod get;

/// CLI for getting information about weather
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Configure the provider and API key for it
    Configure {
        /// Provider name (WeatherApi, OpenWeather)
        #[clap(short, long)]
        provider: String,

        /// API key for the provider, during the first configuration is necessary
        #[clap(short, long)]
        api_key: Option<String>,
    },

    /// Get weather information for the address on the date
    Get {
        /// Name of city
        #[clap(short, long)]
        address: String,

        /// Date [default: now]
        #[clap(short, long)]
        date: Option<String>,
    },
}

/// Processing action for each command
impl Command {
    pub fn process(&self) -> Result<()> {
        match self {
            Command::Configure { provider, api_key } => {
                set_provider::<JsonLoader>(provider, api_key.as_deref(), CONFIG_PATH)?
            }
            Command::Get { address, date } => {
                get::<JsonLoader>(address, date.as_deref(), CONFIG_PATH)?
            }
        }

        Ok(())
    }
}
