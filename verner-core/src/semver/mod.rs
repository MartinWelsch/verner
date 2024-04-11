use std::{borrow::Cow, fmt::{Display, Write}, rc::Rc};

use regex::Regex;

use crate::VersionOp;

pub mod releaseflow;


lazy_static::lazy_static!
{
    static ref SEMVER_REGEX: Regex = Regex::new(r"^(?<major>\d+)\.(?<minor>\d+)(?:\.(?<patch>\d+))?(?:-(?<tag>[^\.]+)(?:\.(?<build>\d+))?)?$").unwrap();
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct SemVersion
{
    major: u32,
    minor: u32,
    patch: u32,
    build: u32,
    tag: Option<Rc<Box<String>>>
}

impl SemVersion
{
    pub fn parse(s: &str) -> Option<Self>
    {
        if let Some(captures) = SEMVER_REGEX.captures(s)
        {
            return Some(Self
            {
                major: captures["major"].parse().unwrap(),
                minor: captures["minor"].parse().unwrap(),
                patch: captures.name("patch").map_or(0, |s| s.as_str().parse().unwrap()),
                build: captures.name("build").map_or(0, |s| s.as_str().parse().unwrap()),
                tag: captures.name("tag").map(|s| Rc::new(Box::new(s.as_str().to_string())))
            })
        }

        None
    }

    pub fn tag(&self, tag: &str) -> SemVersion
    {
        let mut v = self.clone();
        v.tag = Some(Rc::new(Box::new(tag.to_string())));
        v
    }
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

impl PartialOrd for SemVersion
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>
    {
        if self.major > other.major
        {
            return Some(std::cmp::Ordering::Greater);
        }
        
        if self.major < other.major
        {
            return Some(std::cmp::Ordering::Less)
        }

        
        if self.minor > other.minor
        {
            return Some(std::cmp::Ordering::Greater);
        }
        
        if self.minor < other.minor
        {
            return Some(std::cmp::Ordering::Less)
        }
        

        if self.patch > other.patch
        {
            return Some(std::cmp::Ordering::Greater);
        }
        
        if self.patch < other.patch
        {
            return Some(std::cmp::Ordering::Less)
        }

        if self.tag != other.tag
        {
            return None;
        }

        
        if self.build > other.build
        {
            return Some(std::cmp::Ordering::Greater);
        }
        
        if self.build < other.build
        {
            return Some(std::cmp::Ordering::Less)
        }

        
        Some(std::cmp::Ordering::Equal)
    }
}