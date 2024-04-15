use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct RawConfig
{
    pub git: verner_git::RawConfig,
}