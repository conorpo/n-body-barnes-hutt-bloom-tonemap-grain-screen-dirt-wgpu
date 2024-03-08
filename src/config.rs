use serde::{Deserialize};

#[derive(Deserialize)]
pub struct RuntimeSettings {
    width: u32,
    height: u32,
}

#[derive(Deserialize)]
pub struct Config {
    runtime_settings: RuntimeSettings,
    sim_settings: SimSettings,
}

impl Config {
    pub fn default() -> Self {
        Self {
            runtime_settings: RuntimeSettings::default(),
        }
    }

    pub fn new() -> Self {
        let config = r#"
        {
            "window_settings": {
                "width": 800,
                "height": 600,
                "title": "N-Body Simulation"
            }
        }
        "#;
        toml::from_str(config).unwrap_or(Config::default())
    }
}