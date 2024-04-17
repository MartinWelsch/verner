use std::{fs, path::{Path, PathBuf}, process::ExitCode};

use anyhow::bail;
use clap::{Parser, Subcommand};
use config::RawConfig;
use console::Console;
use path_absolutize::Absolutize;
use verner_core::output::LogLevel;
use verner_git::cli::ConfigPreset;

mod console;
mod config;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args
{
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short, long, default_value = ".verner.yml")]
    config_file: PathBuf,

    #[command(subcommand)]
    command: Subcommands
}

#[derive(Subcommand, Debug)]
enum Subcommands
{
    Git(verner_git::cli::Args),
    Init(InitArgs)
}

#[derive(Parser, Debug)]
struct InitArgs
{
    #[command(subcommand)]
    r#type: InitType,
}

#[derive(Debug, Subcommand, Clone)]
enum InitType
{
    Git{ preset: ConfigPreset }
}

fn main() -> ExitCode 
{
    let args = Args::parse();
    let console = Console::default();

    match run(&console, args)
    {
        Ok(_) => {
            console.user_line(LogLevel::Success, "command successful");
            ExitCode::SUCCESS
        },
        Err(err) =>
        {
            console.user_line(LogLevel::Error, format!("an error occurred: {err}"));
            ExitCode::FAILURE
        },
    }
}

fn read_config(path: &Path) -> anyhow::Result<config::RawConfig>
{
    if !path.exists()
    {
        bail!("config file does not exist: {}", path.to_string_lossy());
    }
    let config_text = fs::read_to_string(path)?;
    let config: config::RawConfig = serde_yaml::from_str(&config_text)?;

    Ok(config)
}

fn run(console: &Console, args: Args) -> anyhow::Result<()>
{
    let cwd = args.path.absolutize()?;
    let config_path = if args.config_file.is_absolute() { args.config_file } else { cwd.join(args.config_file) };
    let config_path = config_path.absolutize()?;

    match args.command
    {
        Subcommands::Git(git) => 
        {

            let config = if let Some(ref preset) = git.config_preset { RawConfig { git: verner_git::preset_config(preset)? } } else { read_config(&config_path)? };
            let version = verner_git::solve(console, &cwd, config.git, git)?;
            
            console.user_line(LogLevel::Info, format!("Version: {version}"));
            console.output(version);
        },
        Subcommands::Init(init) => 
        {

            if config_path.exists()
            {
                console.user_line(LogLevel::Error, format!("cannot overwrite existing config file"));
                bail!("file {} already exists", config_path.to_string_lossy());
            }

            let config = match init.r#type
            {
                InitType::Git{ ref preset } => config::RawConfig
                {
                    git: verner_git::preset_config(preset)?
                },
            };

            fs::write(&config_path, serde_yaml::to_string(&config)?)?;
            console.user_line(LogLevel::Success, format!("initialized configuration to {}", config_path.to_string_lossy()));

        },
    };


    Ok(())
}