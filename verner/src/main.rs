use colored::Colorize;
use verner::{version::Detective, config};

mod console;

fn main()
{
    let config = config::from_yaml("
git:

    ").unwrap();
    let detective = Detective::new(&config);
    let mut git_clues = verner::git::search_clues(&config).unwrap();
    let mut clue_walker = git_clues.walk().unwrap();
    let version = detective.find_version(&mut clue_walker).unwrap();

    console::say(&format!("version: {}", version.to_string().green()));
}