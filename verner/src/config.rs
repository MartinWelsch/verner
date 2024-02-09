use serde::{Deserialize, Serialize};

use anyhow::Result;

#[derive(Serialize, Deserialize)]
pub struct Config
{
    #[cfg(feature = "git")]
    pub git: crate::git::Config
}

pub fn from_yaml(yml: &str) -> Result<Config>
{
    Ok(serde_yaml::from_str::<Config>(yml)?)
}
