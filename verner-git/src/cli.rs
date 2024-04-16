use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
pub struct Args
{
    #[arg(long = "preset", short = 'p', default_value = None)]
    pub config_preset: Option<ConfigPreset>,

    #[arg(long = "local", default_value_t = false)]
    pub use_local: bool
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ConfigPreset
{
    Releaseflow
}