use std::path::PathBuf;

use clap::Parser;


#[derive(Debug, Parser)]
pub struct Args
{
    pub path: Option<PathBuf>,
}