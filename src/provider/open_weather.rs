use chrono::{Local, NaiveDate, NaiveDateTime};
use eyre::Result;
use thiserror::Error;

use super::{ProviderIR, WeatherProvider};
use crate::forecast::Forecast;

/// Provider for WeatherApi
pub struct OpenWeather {
    api_key: String,
}

/// Cast from intermediate representation
impl From<ProviderIR> for OpenWeather {
    fn from(value: ProviderIR) -> Self {
        OpenWeather { api_key: value.key }
    }
}

/// Wrapper for casting into Forecast
struct OpenWeatherJson<'a> {
    body: serde_json::Value,
    forecast_date: Option<&'a NaiveDate>,
}

/// Errors for OpenWeather
#[derive(Error, Debug)]
pub enum OpenWeatherError {
    #[error("Bad response with error '{error}'")]
    BadResponse { error: reqwest::Error },
    #[error("Something wrong with {field} in json")]
    InvalidJsonFormat { field: String },
    #[error("Json is invalid with error '{error}'")]
    InvalidJson { error: reqwest::Error },
    #[error("Forecast is not found for date {date}")]
    ForecastNotFound { date: String },
    #[error("Forecast with provider OpenWeather is not available for {date}")]
    ForecastNotAvailable { date: String },
}

impl WeatherProvider for OpenWeather {
    fn get_weather(&self, address: &str, date: Option<&NaiveDate>) -> Result<Forecast> {
        let res: OpenWeatherJson = OpenWeatherJson {
            body: reqwest::blocking::get(match date {
                Some(date) => {
                    let now = Local::now().date_naive();

                    let days = (*date - now).num_days();

                    //WeatherApi can get forecast up to 5 days after now
                    if !(1..=5).contains(&days) {
                        return Err(OpenWeatherError::ForecastNotAvailable {
                            date: date.format("%d.%m.%Y").to_string(),
                        }
                        .into());
                    }

                    format!(
                        // API for forecast up to 5 days
                        "https://api.openweathermap.org/data/2.5/forecast?q={address}&appid={key}",
                        key = self.api_key,
                    )
                }
                None => format!(
                    // API for current weather
                    "https://api.openweathermap.org/data/2.5/weather?q={address}&appid={key}",
                    key = self.api_key
                ),
            })
            .map_err(|error| OpenWeatherError::BadResponse { error })?
            .json()
            .map_err(|error| OpenWeatherError::InvalidJson { error })?,
            forecast_date: date,
        };

        res.into_forecast()
    }
}

impl<'a> OpenWeatherJson<'a> {
    /// Parse JSON for current weather
    pub fn into_forecast(self) -> Result<Forecast> {
        Ok(match self.forecast_date {
            // Parse JSON for current weather
            None => Forecast {
                temp: self
                    .body
                    .get("main")
                    .and_then(|v| v.get("temp"))
                    .and_then(|v| v.as_f64())
                    .map(kelvin_to_celsius)
                    .ok_or(OpenWeatherError::InvalidJsonFormat {
                        field: "main:temp".to_string(),
                    })?,

                condition: self
                    .body
                    .get("weather")
                    .and_then(|v| v.as_array())
                    .and_then(|v| v.last())
                    .and_then(|v| v.get("main"))
                    .and_then(|v| v.as_str())
                    .ok_or(OpenWeatherError::InvalidJsonFormat {
                        field: "weather:main".to_string(),
                    })?
                    .to_owned(),
            },

            // Parse JSON for forecast
            Some(date) => {
                let forecast = self
                    .body
                    .get("list")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|v| {
                        v.iter().find(|v| {
                            if let Some(v) = v
                                .get("dt")
                                .and_then(|v| v.as_i64())
                                .and_then(|v| NaiveDateTime::from_timestamp_millis(v * 1000))
                            {
                                v.date() == *date
                            } else {
                                false
                            }
                        })
                    })
                    .ok_or(OpenWeatherError::ForecastNotFound {
                        date: date.format("%d.%m.%Y").to_string(),
                    })?;

                Forecast {
                    temp: forecast
                        .get("main")
                        .and_then(|v| v.get("temp"))
                        .and_then(|v| v.as_f64())
                        .map(kelvin_to_celsius)
                        .ok_or(OpenWeatherError::InvalidJsonFormat {
                            field: "main:temp".to_string(),
                        })?,

                    condition: forecast
                        .get("weather")
                        .and_then(|v| v.as_array())
                        .and_then(|v| v.last())
                        .and_then(|v| v.get("main"))
                        .and_then(|v| v.as_str())
                        .ok_or(OpenWeatherError::InvalidJsonFormat {
                            field: "weather:main".to_string(),
                        })?
                        .to_owned(),
                }
            }
        })
    }
}

fn kelvin_to_celsius(kelvin: f64) -> f64 {
    ((kelvin - 273.15) * 100.0).round() / 100.0
}
