use std::fmt::{Display, Write};

use crate::VersionOp;

pub mod releaseflow;

#[derive(Clone, Default)]
pub struct SemVersion
{
    major: u32,
    minor: u32,
    patch: u32,
    build: u32,
    tag: Option<String>
}

#[derive(Clone)]
pub enum SemVersionInc
{
    Major(u32),
    Minor(u32),
    Patch(u32),
    Build(u32)
}

impl VersionOp<SemVersionInc> for SemVersion
{
    fn inc(&mut self, i: &SemVersionInc)
    {
        match i
        {
            SemVersionInc::Major(major) =>
            {
                self.major += major;
                self.minor = 0;
                self.patch = 0;
                self.build = 0;
            },
            SemVersionInc::Minor(minor) => 
            {
                self.minor += minor;
                self.patch = 0;
                self.build = 0;
            },
            SemVersionInc::Patch(patch) => 
            {
                self.patch += patch;
                self.build = 0;
            },
            SemVersionInc::Build(build) => 
            {
                self.build += build;
            },
        }
    }
}

impl Display for SemVersion
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(&self.major.to_string())?;
        f.write_char('.')?;
        f.write_str(&self.minor.to_string())?;
        f.write_char('.')?;
        f.write_str(&self.patch.to_string())?;

        if let Some(ref tag) = self.tag
        {
            f.write_char('-')?;
            f.write_str(tag)?;
        }

        if self.build > 0
        {
            f.write_char('.')?;
            f.write_str(&self.build.to_string())?;
        }

        Ok(())
    }
}