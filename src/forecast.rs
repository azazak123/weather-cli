use std::fmt::Display;

/// Representation of information about weather
pub struct Forecast {
    pub temp: f64,
    pub condition: String,
}

/// Pretty print for Forecast
impl Display for Forecast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Temperature: {}", self.temp)?;
        writeln!(f, "Condition: {}", self.condition)
    }
}
