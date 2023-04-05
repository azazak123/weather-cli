use chrono::{Local, NaiveDate};
use eyre::Result;
use thiserror::Error;

use super::{ProviderIR, WeatherProvider};
use crate::forecast::Forecast;

/// Provider for WeatherApi
pub struct WeatherApi {
    api_key: String,
}

/// Cast from intermediate representation
impl From<ProviderIR> for WeatherApi {
    fn from(value: ProviderIR) -> Self {
        WeatherApi { api_key: value.key }
    }
}

/// Wrapper for casting into Forecast
struct WeatherApiJson<'a> {
    body: serde_json::Value,
    forecast_date: Option<&'a NaiveDate>,
}

/// Errors for WeatherApi
#[derive(Error, Debug)]
pub enum WeatherApiError {
    #[error("Bad response with error '{error}'")]
    BadResponse { error: reqwest::Error },
    #[error("Something wrong with {field} in json")]
    InvalidJsonFormat { field: String },
    #[error("Json is invalid with error '{error}'")]
    InvalidJson { error: reqwest::Error },
    #[error("Forecast is not found for date {date}")]
    ForecastNotFound { date: String },
    #[error("Forecast with provider WeatherApi is not available for {date}")]
    ForecastNotAvailable { date: String },
}

impl WeatherProvider for WeatherApi {
    fn get_weather(&self, address: &str, date: Option<&NaiveDate>) -> Result<Forecast> {
        let res: WeatherApiJson = WeatherApiJson {
            body: reqwest::blocking::get(match date {
                Some(date) => {
                    let now = Local::now().date_naive();

                    // WeatherApi forecast give current weather if days = 1, so we should use offset
                    let days = (*date - now).num_days() + 1;

                    // WeatherApi can get forecast up to 13 days after now
                    if !(2..=14).contains(&days) {
                        return Err(WeatherApiError::ForecastNotAvailable { date: date.format("%d.%m.%Y").to_string() }.into());
                    }

                    format!(
                        // API for forecast up to 13 days
                        "http://api.weatherapi.com/v1/forecast.json?key={key}&q={address}&days={days}&aqi=no&alerts=no",
                        key = self.api_key,
                    )
                }

                None => format!(
                    // API for current weather
                    "http://api.weatherapi.com/v1/current.json?key={key}&q={address}&aqi=no",
                    key = self.api_key
                ),
            })
            .map_err(|error| WeatherApiError::BadResponse { error })?
            .json()
            .map_err(|error| WeatherApiError::InvalidJson { error })?,

            forecast_date: date,
        };

        res.into_forecast()
    }
}

impl<'a> WeatherApiJson<'a> {
    /// Cast WeatherApiJson to Forecast
    pub fn into_forecast(self) -> Result<Forecast> {
        Ok(match self.forecast_date {
            // Parse JSON for current weather
            None => {
                let current = self.body.get("current");

                Forecast {
                    temp: current
                        .and_then(|v| v.get("temp_c"))
                        .and_then(|v| v.as_f64())
                        .ok_or(WeatherApiError::InvalidJsonFormat {
                            field: "current:temp_c".to_string(),
                        })?,

                    condition: current
                        .and_then(|v| v.get("condition"))
                        .and_then(|v| v.get("text"))
                        .and_then(|v| v.as_str())
                        .ok_or(WeatherApiError::InvalidJsonFormat {
                            field: "current:condition:text".to_string(),
                        })?
                        .to_owned(),
                }
            }

            // Parse JSON for forecast
            Some(date) => {
                let forecast = self
                    .body
                    .get("forecast")
                    .and_then(|v| v.get("forecastday"))
                    .and_then(serde_json::Value::as_array)
                    .and_then(|v| {
                        v.iter().find(|v| {
                            if let Some(v) = v.get("date").and_then(|v| v.as_str()) {
                                *v == date.format("%Y-%m-%d").to_string()
                            } else {
                                false
                            }
                        })
                    })
                    .ok_or(WeatherApiError::ForecastNotFound {
                        date: date.format("%d.%m.%Y").to_string(),
                    })?;

                Forecast {
                    temp: forecast
                        .get("day")
                        .and_then(|v| v.get("avgtemp_c"))
                        .and_then(|v| v.as_f64())
                        .ok_or(WeatherApiError::InvalidJsonFormat {
                            field: "day:temp_c".to_string(),
                        })?,

                    condition: forecast
                        .get("day")
                        .and_then(|v| v.get("condition"))
                        .and_then(|v| v.get("text"))
                        .and_then(|v| v.as_str())
                        .ok_or(WeatherApiError::InvalidJsonFormat {
                            field: "day:condition:text".to_string(),
                        })?
                        .to_owned(),
                }
            }
        })
    }
}
