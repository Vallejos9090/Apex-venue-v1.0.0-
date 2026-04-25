use clap::Parser;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ConfigError { #[error("Missing env var: {0}")] MissingEnv(String) }
#[derive(Parser, Debug)]
#[command(name = "apex-venue")]
pub struct Config {
        #[arg(long)] pub kafka_brokers: String,
            #[arg(long, default_value = "/data/engine.wal")] pub wal_path: String,
                #[arg(long)] pub db_url: String,
                    #[arg(long, default_value = "64")] pub batch_size: usize,
                        #[arg(long, default_value = "apex-venue")] pub service_name: String,
}
impl Config { pub fn from_env() -> Result<Self, ConfigError> { Ok(Self::parse()) } }
