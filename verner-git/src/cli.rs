use std::path::PathBuf;

use clap::{Parser, ValueEnum};



#[derive(Debug, Parser)]
pub struct Args
{
    #[arg(long = "preset", short = 'p', default_value = None)]
    pub config_preset: Option<ConfigPreset>
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ConfigPreset
{
    Releaseflow
}