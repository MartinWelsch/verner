use std::fmt::Display;

use inline_colorization::*;
use verner_core::output::{ConsoleWriter, LogLevel};



#[derive(Default)]
pub struct Console {}

impl Console
{
    pub fn user_line<D: Display>(&self, level: LogLevel, d: D)
    {
        let indicator_color = match level
        {
            LogLevel::Info => color_bright_blue,
            LogLevel::Warning => color_yellow,
            LogLevel::Error => color_red,
            LogLevel::Success => color_bright_green
        };

        let indicator_symbol = match level
        {
            LogLevel::Info => "🛈",
            LogLevel::Success => "✔",
            LogLevel::Warning => "⚠",
            LogLevel::Error => "☠",
        };

        let level_text = match level
        {
            LogLevel::Info => "INFO",
            LogLevel::Success => "OK",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERR",
        };

        eprintln!("{indicator_color}{indicator_symbol} [{color_cyan}verner{indicator_color}] {level_text:>4}: {color_reset}{d}");
    }

    pub fn output<D: Display>(&self, d: D)
    {
        println!("{d}");
    }
}


impl ConsoleWriter for Console
{
    fn user_line<D: Display>(&self, level: LogLevel, d: D) { self.user_line(level, d); }

    fn output<D: Display>(&self, d: D) { self.output(d); }
}