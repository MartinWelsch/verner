
extern crate semver;

#[cfg(feature = "git")]
pub mod git;

pub mod walker;
pub mod version;
pub mod config;



pub fn parse_config(yml: &str)
{

}