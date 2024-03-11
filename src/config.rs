use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub window_config: WindowConfig,
    pub sim_config: SimConfig,
}

#[derive(Deserialize, Debug)]
pub struct WindowConfig {
    pub title: String,
    pub size: [u32; 2],
}

#[derive(Deserialize, Debug)]
pub struct SimConfig {
    pub desired_maximum_frame_latency: u32,
    pub star_count: u32,
    pub arm_count: u32,
    pub galaxy_radius: f32,
    pub spiralness: f32,
    pub noise_scale: f32,
}

const CONFIG_DIR: &str = "./config/";

impl Config {
    pub fn get() -> Self {
        let args: Vec<String> = std::env::args().collect();

        let config_path = CONFIG_DIR.to_string() + (if args.len() > 1 {
            &args[1]
        } else {
            "default.toml"
        });

        let config_str = std::fs::read_to_string(config_path).expect("Failed to read config file");

        toml::from_str(&config_str).expect("Failed to parse config file")
    }
}