use std::fmt::Display;

pub enum LogLevel
{
    Info,
    Success,
    Warning,
    Error
}

pub trait ConsoleWriter
{
    fn user_line<D: Display>(&self, level: LogLevel, d: D);
    fn output<D: Display>(&self, d: D);
}