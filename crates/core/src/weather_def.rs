use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct WeatherDef {
    pub name: String,
    pub weight: u32,
    pub duration: [u32; 2],
    pub visibility: f32,
    #[serde(default)]
    pub modifiers: HashMap<String, i32>,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub fn load_weather_def(toml_str: &str) -> Option<WeatherDef> {
    toml::from_str::<WeatherDef>(toml_str).ok()
}

/// Load all weather definitions from the bundled TOML files.
/// Paths are relative to the crate root (crates/core/).
pub fn load_all_weathers() -> Vec<WeatherDef> {
    let mut weathers = Vec::new();
    macro_rules! load {
        ($path:literal) => {
            if let Some(w) = load_weather_def(include_str!($path)) {
                weathers.push(w);
            }
        };
    }
    load!("../../../assets/config/weather/weather_clear.toml");
    load!("../../../assets/config/weather/weather_cloudy.toml");
    load!("../../../assets/config/weather/weather_fog.toml");
    load!("../../../assets/config/weather/weather_rain.toml");
    load!("../../../assets/config/weather/weather_storm.toml");
    load!("../../../assets/config/weather/weather_snow.toml");
    load!("../../../assets/config/weather/weather_sandstorm.toml");
    load!("../../../assets/config/weather/weather_ashfall.toml");
    weathers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_clear_weather() {
        let toml = r#"
name = "Clear"
weight = 30
duration = [8, 25]
visibility = 1.0
"#;
        let def = load_weather_def(toml).unwrap();
        assert_eq!(def.name, "Clear");
        assert_eq!(def.weight, 30);
        assert_eq!(def.duration, [8, 25]);
        assert_eq!(def.visibility, 1.0);
        assert!(def.modifiers.is_empty());
        assert!(def.tags.is_empty());
    }

    #[test]
    fn test_load_rain_weather() {
        let toml = r#"
name = "Rain"
weight = 15
duration = [5, 15]
visibility = 0.6
modifiers = { moisture = 50 }
tags = ["RAINY"]
"#;
        let def = load_weather_def(toml).unwrap();
        assert_eq!(def.name, "Rain");
        assert_eq!(def.modifiers.get("moisture"), Some(&50));
        assert_eq!(def.tags, vec!["RAINY"]);
    }

    #[test]
    fn test_load_fire_storm_weather() {
        let toml = r#"
name = "FireStorm"
weight = 3
duration = [3, 8]
visibility = 0.4
modifiers = { temperature = 50 }
"#;
        let def = load_weather_def(toml).unwrap();
        assert_eq!(def.name, "FireStorm");
        assert_eq!(def.modifiers.get("temperature"), Some(&50));
    }

    #[test]
    fn test_invalid_toml_returns_none() {
        let result = load_weather_def("not valid toml {{{");
        assert!(result.is_none());
    }

    #[test]
    fn test_all_weather_files_load() {
        let weathers = load_all_weathers();
        assert_eq!(weathers.len(), 8, "all 8 weather TOML files should load");
        let names: Vec<&str> = weathers.iter().map(|w| w.name.as_str()).collect();
        assert!(names.contains(&"Clear"));
        assert!(names.contains(&"Rain"));
        assert!(names.contains(&"Snow"));
        assert!(names.contains(&"Storm"));
        assert!(names.contains(&"Fog"));
        assert!(names.contains(&"Cloudy"));
        assert!(names.contains(&"Sandstorm"));
        assert!(names.contains(&"AshFall"));
    }
}
