use std::fmt::Display;

use inline_colorization::*;
use verner_core::output::{ConsoleWriter, LogLevel};



#[derive(Default)]
pub struct Console
{
    pub min_level: LogLevel
}

impl Console
{
    pub fn user_line<D: Display>(&self, level: LogLevel, d: D)
    {
        if level < self.min_level
        {
            return;
        }

        let indicator_color = match level
        {
            LogLevel::Trace => color_bright_black,
            LogLevel::Info => color_bright_blue,
            LogLevel::Warning => color_yellow,
            LogLevel::Error => color_red,
            LogLevel::Success => color_bright_green
        };

        let indicator_symbol = match level
        {
            LogLevel::Trace => "✎",
            LogLevel::Info => "🛈",
            LogLevel::Success => "✔",
            LogLevel::Warning => "⚠",
            LogLevel::Error => "☠",
        };

        let level_text = match level
        {
            LogLevel::Trace => "TRCE",
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