use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
pub struct Args
{
    #[arg(long = "preset", short = 'p', default_value = None)]
    pub config_preset: Option<ConfigPreset>,

    #[arg(long = "local", default_value_t = false)]
    pub use_local: bool,

    #[arg(long = "branch", short = 'b', default_value = None)]
    pub branch_name: Option<String>,

    #[arg(long = "git-dir", default_value = None)]
    pub git_dir: Option<PathBuf>
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ConfigPreset
{
    Releaseflow
}