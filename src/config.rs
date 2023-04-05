use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::provider::ProviderType;

/// Config that consists default provider and API keys
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub default: Option<ProviderType>,
    pub keys: HashMap<ProviderType, String>,
}
