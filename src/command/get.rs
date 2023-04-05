use chrono::{Local, NaiveDate, ParseError};
use eyre::Result;
use thiserror::Error;

use crate::{provider::WeatherProvider, provider_loader::ProviderLoader};

/// Errors for action get
#[derive(Error, Debug)]
pub enum GettingError {
    #[error("Date {date} is in unsupported format with error '{error}'")]
    InvalidDate { error: ParseError, date: String },
    #[error("Date {date} should be >= now")]
    PastDate { date: String },
}

/// Get weather information for provided address and date (none => now)
pub fn get<Loader: ProviderLoader>(
    address: &str,
    date: Option<&str>,
    loader_args: Loader::ConstructorArg<'_>,
) -> Result<()> {
    let mut loader = Loader::new(loader_args)?;
    loader.load_config()?;

    let provider = loader.get_default_provider()?;

    let now = Local::now().naive_local().date();

    // Parse and validate provided date
    let date = date
        .map(|date_str| {
            NaiveDate::parse_from_str(date_str, "%d.%m.%Y").map_err(|error| {
                GettingError::InvalidDate {
                    error,
                    date: date_str.to_owned(),
                }
            })
        })
        .transpose()?
        .map(|date| {
            if date < now {
                Err(GettingError::PastDate {
                    date: date.format("%d.%m.%Y").to_string(),
                })
            } else {
                Ok(date)
            }
        })
        .transpose()?
        .filter(|date| *date != now);

    let weather = provider.get_weather(address, date.as_ref())?;

    println!(
        "Weather information for {address} on {date}:\n{weather}",
        date = date.unwrap_or(now).format("%d.%m.%Y")
    );

    Ok(())
}
