use std::{fs, path::{Path, PathBuf}};

use anyhow::bail;
use clap::{Parser, Subcommand, ValueEnum};
use console::Console;
use path_absolutize::Absolutize;
use verner_core::output::LogLevel;

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
    command: Option<Subcommands>
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
    #[arg()]
    r#type: InitType,
}

#[derive(Debug, ValueEnum, Clone)]
enum InitType
{
    Git
}

fn main()
{
    let args = Args::parse();
    let console = Console::default();

    match run(&console, args)
    {
        Ok(_) => console.user_line(LogLevel::Success, "command successful"),
        Err(err) =>
        {
            console.user_line(LogLevel::Error, format!("an error occurred: {err}"))
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
    let config_path = cwd.join(args.config_file);
    let config_path = config_path.absolutize()?;

    match args.command.unwrap_or_else(|| Subcommands::Git(verner_git::cli::Args::parse()))
    {
        Subcommands::Git(git) => 
        {
            let config = read_config(&config_path)?;
            let version = verner_git::solve(&cwd, config.git, git)?;
            
            console.user_line(LogLevel::Info, format!("Version: {version}"));
            console.output(version);
        },
        Subcommands::Init(init) => 
        {

            if cwd.exists()
            {
                console.user_line(LogLevel::Error, format!("cannot overwrite existing config file"));
                bail!("file {} already exists", cwd.to_string_lossy());
            }

            let config = match init.r#type
            {
                InitType::Git => config::RawConfig
                {
                    git: verner_git::default_config()?
                },
            };

            fs::write(&cwd, serde_yaml::to_string(&config)?)?;
            console.user_line(LogLevel::Success, format!("initialized configuration to {}", cwd.to_string_lossy()));

        },
    };


    Ok(())
}