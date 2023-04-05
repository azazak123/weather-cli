use chrono::NaiveDate;
use enum_dispatch::enum_dispatch;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

use self::open_weather::OpenWeather;
use self::weather_api::WeatherApi;
use crate::forecast::Forecast;

mod open_weather;
mod weather_api;

/// Trait for all weather providers
#[enum_dispatch(Provider)]
pub trait WeatherProvider: From<ProviderIR> {
    fn get_weather(&self, address: &str, date: Option<&NaiveDate>) -> Result<Forecast>;
}

/// Intermediate representation of provider
pub struct ProviderIR {
    pub provider: ProviderType,
    pub key: String,
}

/// Macro for providers registration
macro_rules! register {
    (Provider: $provider:ident,
        ProviderType:$provider_type:ident,
        ProviderTypeMap: $provider_type_map:ident,
        Providers:[$($i:ident),*]) => {

        /// Sum of types of providers
        #[enum_dispatch]
        pub enum $provider {
            $( $i, )*
        }

        /// Provider types
        #[derive(PartialEq, Eq, Serialize, Deserialize, Hash, Debug, Clone, Copy)]
        pub enum $provider_type {
            $( $i, )*
        }

        /// Provider name mapping to the provider type
        pub static $provider_type_map: LazyLock<HashMap<&str, ProviderType>> = LazyLock::new(|| {
            let mut map = HashMap::new();

            $(map.insert(stringify!($i), ProviderType::$i);)*

            map
        });

        /// Implementation of casting intermediate representation to corresponding provider
        impl From<ProviderIR> for Provider {
           fn from(value: ProviderIR) -> Self {
                match value.provider {
                     $(ProviderType::$i => $i::from(value).into(),)*
                }
            }
        }
    };
}

// Provider registration
register!(
    Provider: Provider,
    ProviderType: ProviderType,
    ProviderTypeMap: PROVIDER_TYPE_MAP,
    Providers: [WeatherApi, OpenWeather]
);
