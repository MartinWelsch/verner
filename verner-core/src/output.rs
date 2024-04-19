use std::fmt::Display;

#[derive(PartialEq, PartialOrd)]
pub enum LogLevel
{
    Trace,
    Info,
    Success,
    Warning,
    Error
}

impl Default for LogLevel
{
    fn default() -> Self { LogLevel::Info }
}

pub trait ConsoleWriter
{
    fn user_line<D: Display>(&self, level: LogLevel, d: D);
    fn output<D: Display>(&self, d: D);
}