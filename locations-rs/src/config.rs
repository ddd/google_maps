use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub options: OptionsConfig,
    pub tile_generation: TileGenerationConfig,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub uri: String,
    pub keyspace: String,
    pub workers: usize,
}


#[derive(Debug, Deserialize)]
pub struct OptionsConfig {
    pub output: String,
    pub max_retries: usize,
    pub fetchers: usize,
}

#[derive(Debug, Deserialize)]
pub struct TileGenerationConfig {
    pub min_x: usize,
    pub max_x: usize,
    pub min_y: usize,
    pub max_y: usize,
    pub reverse: bool,
    pub zoom: usize,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}